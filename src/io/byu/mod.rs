//! BYU Movie format reader and writer for vtk-rs.
//!
//! The BYU format stores polygonal meshes with a simple header:
//! `num_parts num_vertices num_polygons num_edges`
//! followed by part boundaries, vertices (x y z per line), and
//! polygon connectivity (negative index terminates each polygon).

use crate::data::{CellArray, Points, PolyData};
use std::io::{BufRead, Write};

/// Write a PolyData mesh in BYU format.
pub fn write_byu<W: Write>(w: &mut W, mesh: &PolyData) -> std::io::Result<()> {
    let n_pts = mesh.points.len();
    let n_polys = mesh.polys.num_cells();
    // Count total edges (sum of polygon sizes)
    let mut n_edges = 0;
    for cell in mesh.polys.iter() {
        n_edges += cell.len();
    }

    // Header: parts vertices polygons edges
    writeln!(w, "1 {n_pts} {n_polys} {n_edges}")?;
    // Part range (1-based)
    writeln!(w, "1 {n_polys}")?;

    // Vertices (x y z)
    for i in 0..n_pts {
        let p = mesh.points.get(i);
        writeln!(w, "{:e} {:e} {:e}", p[0], p[1], p[2])?;
    }

    // Polygons (1-based indices, last one negative)
    for cell in mesh.polys.iter() {
        let n = cell.len();
        for (i, &pid) in cell.iter().enumerate() {
            let idx = pid + 1; // 1-based
            if i == n - 1 {
                write!(w, "{}", -idx)?;
            } else {
                write!(w, "{idx} ")?;
            }
        }
        writeln!(w)?;
    }

    Ok(())
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
    let _n_edges = next_i64(&mut tokens, "number of edges")?;

    if n_parts < 1 || n_verts < 1 || n_polys < 1 {
        return Err("Bad MOVIE.BYU file".into());
    }
    let n_parts = n_parts as usize;
    let n_verts = n_verts as usize;
    let n_polys = n_polys as usize;

    // Read vertices
    for _ in 0..n_parts {
        let _part_start = next_i64(&mut tokens, "part start")?;
        let _part_end = next_i64(&mut tokens, "part end")?;
    }
    let mut points = Points::<f64>::new();
    for _ in 0..n_verts {
        points.push([
            next_f64(&mut tokens, "point x")?,
            next_f64(&mut tokens, "point y")?,
            next_f64(&mut tokens, "point z")?,
        ]);
    }

    // Read polygon connectivity
    let mut polys = CellArray::new();
    for _ in 0..n_polys {
        let mut current_poly: Vec<i64> = Vec::new();
        loop {
            let value = next_i64(&mut tokens, "polygon connectivity")?;
            if value == 0 {
                return Err("BYU point indices are 1-based and must not be zero".into());
            }
            if value < 0 {
                current_poly.push(-value - 1); // convert to 0-based
                break;
            }
            current_poly.push(value - 1); // convert to 0-based
        }
        if current_poly.is_empty() {
            return Err("empty BYU polygon".into());
        }
        polys.push_cell(&current_poly);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
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
