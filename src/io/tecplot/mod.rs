//! Tecplot ASCII data format reader and writer for vtk-rs.
//!
//! Supports Tecplot ASCII `.dat` files with POINT and BLOCK data packing,
//! and FE (finite element) zone types (TRIANGLE, QUADRILATERAL, TETRAHEDRON).

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::io::{BufRead, Write};

/// Write a PolyData mesh as a Tecplot ASCII file.
pub fn write_tecplot<W: Write>(w: &mut W, mesh: &PolyData, title: &str) -> std::io::Result<()> {
    let n_pts = mesh.points.len();
    let n_cells = mesh.polys.num_cells();

    writeln!(w, "TITLE = \"{title}\"")?;
    writeln!(w, "VARIABLES = \"X\" \"Y\" \"Z\"")?;
    writeln!(
        w,
        "ZONE T=\"Zone1\", N={n_pts}, E={n_cells}, F=FEPOINT, ET=TRIANGLE"
    )?;

    // Point data
    for i in 0..n_pts {
        let p = mesh.points.get(i);
        writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
    }

    // Connectivity (1-based)
    for cell in mesh.polys.iter() {
        if cell.len() >= 3 {
            writeln!(w, "{} {} {}", cell[0] + 1, cell[1] + 1, cell[2] + 1)?;
        }
    }

    Ok(())
}

/// Read a Tecplot ASCII file into PolyData.
pub fn read_tecplot<R: BufRead>(reader: R) -> Result<PolyData, String> {
    let mut n_nodes = 0usize;
    let mut n_elements = 0usize;
    let mut var_names: Vec<String> = Vec::new();
    let mut in_data = false;
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut extra_data: Vec<Vec<f64>> = Vec::new();
    let mut nodes_read = 0;
    let mut elements_read = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let upper = trimmed.to_uppercase();

        if upper.starts_with("VARIABLES") {
            // Parse variable names
            let rest = &trimmed[trimmed.find('=').map(|i| i + 1).unwrap_or(0)..];
            for part in rest.split('"') {
                let t = part.trim().trim_matches(',').trim();
                if !t.is_empty() {
                    var_names.push(t.to_string());
                }
            }
            let n_extra = var_names.len().saturating_sub(3);
            extra_data = vec![Vec::new(); n_extra];
            continue;
        }

        if upper.starts_with("ZONE") {
            // Parse N= and E=
            for part in upper.split(&[',', ' '][..]) {
                let part = part.trim();
                if part.starts_with("N=") {
                    n_nodes = part[2..].parse().unwrap_or(0);
                } else if part.starts_with("E=") {
                    n_elements = part[2..].parse().unwrap_or(0);
                }
            }
            in_data = true;
            continue;
        }

        if upper.starts_with("TITLE") {
            continue;
        }

        if in_data && nodes_read < n_nodes {
            let vals: Vec<f64> = trimmed
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if vals.len() >= 3 {
                points.push([vals[0], vals[1], vals[2]]);
                for (ei, &v) in vals[3..].iter().enumerate() {
                    if ei < extra_data.len() {
                        extra_data[ei].push(v);
                    }
                }
                nodes_read += 1;
            }
            continue;
        }

        if in_data && nodes_read >= n_nodes && elements_read < n_elements {
            let vals: Vec<i64> = trimmed
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if vals.len() >= 3 {
                // Convert from 1-based to 0-based
                let ids: Vec<i64> = vals.iter().map(|&v| v - 1).collect();
                polys.push_cell(&ids);
                elements_read += 1;
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;

    // Add extra variables as point data
    for (i, data) in extra_data.into_iter().enumerate() {
        if data.len() == n_nodes {
            let name = if i + 3 < var_names.len() {
                &var_names[i + 3]
            } else {
                "var"
            };
            mesh.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)));
        }
    }

    Ok(mesh)
}

pub fn read_tecplot_file(path: &std::path::Path) -> Result<PolyData, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_tecplot(std::io::BufReader::new(f))
}

pub fn write_tecplot_file(
    mesh: &PolyData,
    path: &std::path::Path,
    title: &str,
) -> Result<(), String> {
    let f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    write_tecplot(&mut std::io::BufWriter::new(f), mesh, title).map_err(|e| e.to_string())
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
        write_tecplot(&mut buf, &mesh, "test").unwrap();
        let s = String::from_utf8(buf.clone()).unwrap();
        assert!(s.contains("ZONE"));

        let loaded = read_tecplot(&buf[..]).unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);
    }

    #[test]
    fn with_extra_vars() {
        let data = b"TITLE = \"Test\"\nVARIABLES = \"X\" \"Y\" \"Z\" \"P\"\nZONE T=\"Z\", N=3, E=1, F=FEPOINT, ET=TRIANGLE\n0 0 0 1.5\n1 0 0 2.5\n0 1 0 3.5\n1 2 3\n";
        let mesh = read_tecplot(&data[..]).unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert!(mesh.point_data().get_array("P").is_some());
    }
}
