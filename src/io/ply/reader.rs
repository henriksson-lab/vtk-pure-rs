use std::io::BufRead;
use std::path::Path;

use crate::data::{CellArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for Stanford PLY format (ASCII only).
pub struct PlyReader;

impl PlyReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<PolyData, VtkError> {
        let mut lines = reader.lines();
        let mut n_vertices = 0;
        let mut n_faces = 0;

        // Parse header
        let first = read_line(&mut lines)?;
        if first.trim() != "ply" {
            return Err(VtkError::Parse("not a PLY file".into()));
        }

        let mut in_header = true;
        let mut vertex_props: Vec<String> = Vec::new();
        let mut current_element = String::new();

        while in_header {
            let line = read_line(&mut lines)?;
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "format" => {
                    // We only support ASCII
                    if parts.get(1) != Some(&"ascii") {
                        return Err(VtkError::Unsupported(
                            "only ASCII PLY format supported".into(),
                        ));
                    }
                }
                "element" => {
                    if parts.len() >= 3 {
                        current_element = parts[1].to_string();
                        let count: usize = parts[2]
                            .parse()
                            .map_err(|_| VtkError::Parse("invalid element count".into()))?;
                        if current_element == "vertex" {
                            n_vertices = count;
                        } else if current_element == "face" {
                            n_faces = count;
                        }
                    }
                }
                "property" => {
                    if current_element == "vertex" && parts.len() >= 3 {
                        vertex_props.push(parts.last().unwrap().to_string());
                    }
                }
                "end_header" => {
                    in_header = false;
                }
                _ => {} // comment, etc.
            }
        }

        // Determine which columns are x, y, z
        let x_col = vertex_props.iter().position(|p| p == "x").unwrap_or(0);
        let y_col = vertex_props.iter().position(|p| p == "y").unwrap_or(1);
        let z_col = vertex_props.iter().position(|p| p == "z").unwrap_or(2);

        // Read vertices
        let mut points = Points::<f64>::new();
        for _ in 0..n_vertices {
            let line = read_line(&mut lines)?;
            let values: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            let x = values.get(x_col).copied().unwrap_or(0.0);
            let y = values.get(y_col).copied().unwrap_or(0.0);
            let z = values.get(z_col).copied().unwrap_or(0.0);
            points.push([x, y, z]);
        }

        // Read faces
        let mut polys = CellArray::new();
        for _ in 0..n_faces {
            let line = read_line(&mut lines)?;
            let values: Vec<i64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if values.len() >= 4 {
                // First value is vertex count
                let n = values[0] as usize;
                let indices = &values[1..=n];
                polys.push_cell(indices);
            }
        }

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        Ok(pd)
    }
}

fn read_line(
    lines: &mut impl Iterator<Item = Result<String, std::io::Error>>,
) -> Result<String, VtkError> {
    lines
        .next()
        .ok_or_else(|| VtkError::Parse("unexpected end of file".into()))?
        .map_err(VtkError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ply::PlyWriter;

    #[test]
    fn roundtrip() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        PlyWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = PlyReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);

        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-6);
        assert!((p0[1] - 2.0).abs() < 1e-6);
    }
}
