//! Facet file format reader and writer.
//!
//! Simple ASCII format: header line with counts, then vertex coordinates
//! and triangle connectivity (0-based indices).

use crate::data::{CellArray, Points, PolyData};
use std::io::{BufRead, Write};

/// Write PolyData as Facet format.
pub fn write_facet<W: Write>(w: &mut W, mesh: &PolyData) -> std::io::Result<()> {
    let n_pts = mesh.points.len();
    let n_tris = mesh.polys.num_cells();
    writeln!(w, "{n_pts} {n_tris}")?;
    for i in 0..n_pts {
        let p = mesh.points.get(i);
        writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
    }
    for cell in mesh.polys.iter() {
        if cell.len() >= 3 {
            writeln!(w, "{} {} {}", cell[0], cell[1], cell[2])?;
        }
    }
    Ok(())
}

/// Read Facet format into PolyData.
pub fn read_facet<R: BufRead>(reader: R) -> Result<PolyData, String> {
    let mut nums: Vec<f64> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        for tok in line.split_whitespace() {
            if let Ok(v) = tok.parse::<f64>() {
                nums.push(v);
            }
        }
    }
    if nums.len() < 2 {
        return Err("too short".into());
    }
    let n_pts = nums[0] as usize;
    let n_tris = nums[1] as usize;
    let mut idx = 2;
    let mut points = Points::<f64>::new();
    for _ in 0..n_pts {
        if idx + 2 >= nums.len() {
            break;
        }
        points.push([nums[idx], nums[idx + 1], nums[idx + 2]]);
        idx += 3;
    }
    let mut polys = CellArray::new();
    for _ in 0..n_tris {
        if idx + 2 >= nums.len() {
            break;
        }
        polys.push_cell(&[nums[idx] as i64, nums[idx + 1] as i64, nums[idx + 2] as i64]);
        idx += 3;
    }
    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    Ok(mesh)
}

pub fn read_facet_file(path: &std::path::Path) -> Result<PolyData, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_facet(std::io::BufReader::new(f))
}

pub fn write_facet_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    write_facet(&mut std::io::BufWriter::new(f), mesh).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_facet(&mut buf, &mesh).unwrap();
        let loaded = read_facet(&buf[..]).unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);
    }
}
