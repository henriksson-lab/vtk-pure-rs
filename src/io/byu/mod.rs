//! BYU Movie format reader and writer for vtk-rs.
//!
//! The BYU format stores polygonal meshes with a simple header:
//! `num_parts num_vertices num_polygons num_edges`
//! followed by part boundaries, vertices (x y z per line), and
//! polygon connectivity (negative index terminates each polygon).

use crate::data::{CellArray, Points, PolyData};
use std::fmt::Write as FmtWrite;
use std::io::{BufRead, Write as IoWrite};

/// Write a PolyData mesh in BYU format.
pub fn write_byu<W: IoWrite>(w: &mut W, mesh: &PolyData) -> std::io::Result<()> {
    let n_pts = mesh.points.len();
    let n_polys = mesh.polys.num_cells();
    let n_edges = mesh.polys.connectivity_len();

    let mut out = String::with_capacity(estimate_byu_size(n_pts, n_edges, n_polys));
    writeln!(&mut out, "1 {n_pts} {n_polys} {n_edges}").unwrap();
    writeln!(&mut out, "1 {n_polys}").unwrap();

    for p in mesh.points.as_flat_slice().chunks_exact(3) {
        writeln!(&mut out, "{:e} {:e} {:e}", p[0], p[1], p[2]).unwrap();
    }

    for cell in mesh.polys.iter() {
        let n = cell.len();
        for (i, &pid) in cell.iter().enumerate() {
            let idx = pid + 1; // 1-based
            if i == n - 1 {
                write!(&mut out, "{}", -idx).unwrap();
            } else {
                write!(&mut out, "{idx} ").unwrap();
            }
        }
        out.push('\n');
    }

    w.write_all(out.as_bytes())?;
    Ok(())
}

fn estimate_byu_size(n_pts: usize, n_edges: usize, n_polys: usize) -> usize {
    64 + n_pts * 72 + n_edges * 8 + n_polys
}

/// Read a BYU format file into a PolyData mesh.
pub fn read_byu<R: BufRead>(reader: R) -> Result<PolyData, String> {
    let mut input = String::new();
    let mut reader = reader;
    reader
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let mut tokens = input.split_whitespace();

    let n_parts = next_i64(&mut tokens, "number of parts")?;
    let n_verts = next_i64(&mut tokens, "number of vertices")?;
    let n_polys = next_i64(&mut tokens, "number of polygons")?;
    let n_edges = next_i64(&mut tokens, "number of edges")?;

    if n_parts < 1 || n_verts < 1 || n_polys < 1 {
        return Err("Bad MOVIE.BYU file".into());
    }
    let n_parts = n_parts as usize;
    let n_verts = n_verts as usize;
    let n_polys = n_polys as usize;
    let n_edges = usize::try_from(n_edges).map_err(|_| "invalid BYU edge count".to_string())?;

    for _ in 0..n_parts {
        let _part_start = next_i64(&mut tokens, "part start")?;
        let _part_end = next_i64(&mut tokens, "part end")?;
    }
    let mut points = Vec::with_capacity(n_verts * 3);
    for _ in 0..n_verts {
        points.push(next_f64(&mut tokens, "point x")?);
        points.push(next_f64(&mut tokens, "point y")?);
        points.push(next_f64(&mut tokens, "point z")?);
    }

    let mut offsets = Vec::with_capacity(n_polys + 1);
    let mut connectivity = Vec::with_capacity(n_edges);
    offsets.push(0);
    for _ in 0..n_polys {
        let start = connectivity.len();
        loop {
            let value = next_i64(&mut tokens, "polygon connectivity")?;
            if value == 0 {
                return Err("BYU point indices are 1-based and must not be zero".into());
            }
            if value < 0 {
                connectivity.push(-value - 1); // convert to 0-based
                break;
            }
            connectivity.push(value - 1); // convert to 0-based
        }
        if connectivity.len() == start {
            return Err("empty BYU polygon".into());
        }
        offsets.push(connectivity.len() as i64);
    }

    let mut mesh = PolyData::new();
    mesh.points = Points::from_flat_vec(points);
    mesh.polys = CellArray::from_raw(offsets, connectivity);
    Ok(mesh)
}

fn next_i64<'a>(tokens: &mut impl Iterator<Item = &'a str>, what: &str) -> Result<i64, String> {
    let token = tokens.next().ok_or_else(|| format!("missing BYU {what}"))?;
    token
        .parse::<i64>()
        .map_err(|_| format!("invalid BYU {what}: {token}"))
}

fn next_f64<'a>(tokens: &mut impl Iterator<Item = &'a str>, what: &str) -> Result<f64, String> {
    let token = tokens.next().ok_or_else(|| format!("missing BYU {what}"))?;
    token
        .parse::<f64>()
        .map_err(|_| format!("invalid BYU {what}: {token}"))
}

/// Read BYU from file path.
pub fn read_byu_file(path: &std::path::Path) -> Result<PolyData, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_byu(std::io::BufReader::new(file))
}

/// Write BYU to file path.
pub fn write_byu_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    write_byu(&mut std::io::BufWriter::new(file), mesh).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_byu(&mut buf, &mesh).unwrap();
        let loaded = read_byu(&buf[..]).unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);
        let p = loaded.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn roundtrip_two_triangles() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let mut buf = Vec::new();
        write_byu(&mut buf, &mesh).unwrap();
        let loaded = read_byu(&buf[..]).unwrap();
        assert_eq!(loaded.points.len(), 4);
        assert_eq!(loaded.polys.num_cells(), 2);
    }

    #[test]
    fn quad() {
        let mesh = PolyData::from_quads(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        let mut buf = Vec::new();
        write_byu(&mut buf, &mesh).unwrap();
        let loaded = read_byu(&buf[..]).unwrap();
        assert_eq!(loaded.polys.num_cells(), 1);
    }
}
