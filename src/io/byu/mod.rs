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
        writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
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
    let mut all_nums: Vec<f64> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        for token in line.split_whitespace() {
            if let Ok(v) = token.parse::<f64>() {
                all_nums.push(v);
            }
        }
    }

    if all_nums.len() < 4 {
        return Err("BYU file too short".into());
    }

    let _n_parts = all_nums[0] as usize;
    let n_verts = all_nums[1] as usize;
    let _n_polys = all_nums[2] as usize;
    let _n_conn = all_nums[3] as usize;

    // Skip part ranges (2 values per part)
    let mut idx = 4 + _n_parts * 2;

    // Read vertices
    if idx + n_verts * 3 > all_nums.len() {
        return Err("not enough vertex data".into());
    }
    let mut points = Points::<f64>::new();
    for i in 0..n_verts {
        let base = idx + i * 3;
        points.push([all_nums[base], all_nums[base + 1], all_nums[base + 2]]);
    }
    idx += n_verts * 3;

    // Read polygon connectivity
    let mut polys = CellArray::new();
    let mut current_poly: Vec<i64> = Vec::new();
    let remaining = &all_nums[idx..];
    for &val in remaining {
        let ival = val as i64;
        if ival < 0 {
            current_poly.push(-ival - 1); // convert to 0-based
            polys.push_cell(&current_poly);
            current_poly.clear();
        } else {
            current_poly.push(ival - 1); // convert to 0-based
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    Ok(mesh)
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
