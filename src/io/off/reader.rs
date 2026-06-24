use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::io::BufRead;

/// Reader for Object File Format (OFF).
pub struct OffReader<R: BufRead> {
    reader: R,
}

impl<R: BufRead> OffReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read an OFF file and return a PolyData mesh.
    pub fn read(&mut self) -> Result<PolyData, String> {
        let mut lines = Vec::new();
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            let n = self
                .reader
                .read_line(&mut line_buf)
                .map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            let trimmed = line_buf.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            lines.push(trimmed.to_string());
        }

        if lines.is_empty() {
            return Err("empty OFF file".into());
        }

        let mut idx = 0;

        // Parse header
        let header = &lines[idx];
        let has_colors = header.starts_with("COFF") || header.starts_with("coff");
        let has_normals = header.starts_with("NOFF") || header.starts_with("noff");
        let is_off = header == "OFF" || header == "off" || has_colors || has_normals;

        if !is_off {
            // Some OFF files have counts on the same line as "OFF"
            if !header.contains("OFF") && !header.contains("off") {
                return Err(format!("not an OFF file, got: {header}"));
            }
        }

        idx += 1;
        if idx >= lines.len() {
            return Err("unexpected end of OFF file".into());
        }

        // Parse counts: nVertices nFaces nEdges
        let counts: Vec<usize> = lines[idx]
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if counts.len() < 2 {
            return Err(format!("expected vertex/face counts, got: {}", lines[idx]));
        }
        let n_verts = counts[0];
        let n_faces = counts[1];
        idx += 1;

        // Parse vertices
        let mut points = Points::<f64>::new();
        let mut colors: Vec<f64> = Vec::new();

        for i in 0..n_verts {
            if idx + i >= lines.len() {
                return Err(format!("expected {n_verts} vertices, got {i}"));
            }
            let parts: Vec<f64> = lines[idx + i]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() < 3 {
                return Err(format!("vertex {i} has fewer than 3 coordinates"));
            }
            points.push([parts[0], parts[1], parts[2]]);

            if has_colors && parts.len() >= 6 {
                // Colors as floats or ints (0-255)
                let r = if parts[3] > 1.0 {
                    parts[3] / 255.0
                } else {
                    parts[3]
                };
                let g = if parts[4] > 1.0 {
                    parts[4] / 255.0
                } else {
                    parts[4]
                };
                let b = if parts[5] > 1.0 {
                    parts[5] / 255.0
                } else {
                    parts[5]
                };
                colors.push(r);
                colors.push(g);
                colors.push(b);
            }
        }
        idx += n_verts;

        // Parse faces
        let mut polys = CellArray::new();
        for i in 0..n_faces {
            if idx + i >= lines.len() {
                return Err(format!("expected {n_faces} faces, got {i}"));
            }
            let parts: Vec<i64> = lines[idx + i]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.is_empty() {
                continue;
            }
            let n = parts[0] as usize;
            if parts.len() < n + 1 {
                return Err(format!(
                    "face {i}: expected {n} indices, got {}",
                    parts.len() - 1
                ));
            }
            let ids: Vec<i64> = parts[1..=n].to_vec();
            polys.push_cell(&ids);
        }

        let mut mesh = PolyData::new();
        mesh.points = points;
        mesh.polys = polys;

        if !colors.is_empty() && colors.len() == n_verts * 3 {
            let arr = DataArray::from_vec("Colors", colors, 3);
            mesh.point_data_mut().add_array(AnyDataArray::F64(arr));
            mesh.point_data_mut().set_active_scalars("Colors");
        }

        Ok(mesh)
    }
}

/// Read an OFF file from a file path.
pub fn read_off_file(path: &std::path::Path) -> Result<PolyData, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    OffReader::new(reader).read()
}

/// Write an OFF file to a file path.
pub fn write_off_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut writer = std::io::BufWriter::new(file);
    crate::io::off::OffWriter::new(&mut writer)
        .write(mesh)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_simple_off() {
        let data = b"OFF\n3 1 0\n0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n";
        let mut reader = OffReader::new(&data[..]);
        let mesh = reader.read().unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert_eq!(mesh.polys.num_cells(), 1);
    }

    #[test]
    fn read_coff() {
        let data =
            b"COFF\n3 1 0\n0 0 0 255 0 0 255\n1 0 0 0 255 0 255\n0 1 0 0 0 255 255\n3 0 1 2\n";
        let mut reader = OffReader::new(&data[..]);
        let mesh = reader.read().unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert!(mesh.point_data().scalars().is_some());
    }

    #[test]
    fn roundtrip() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        crate::io::off::OffWriter::new(&mut buf)
            .write(&mesh)
            .unwrap();

        let mut reader = OffReader::new(&buf[..]);
        let loaded = reader.read().unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);

        // Check coordinates
        let p = loaded.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-6);
        assert!((p[1] - 2.0).abs() < 1e-6);
        assert!((p[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn quad_roundtrip() {
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
        crate::io::off::OffWriter::new(&mut buf)
            .write(&mesh)
            .unwrap();

        let loaded = OffReader::new(&buf[..]).read().unwrap();
        assert_eq!(loaded.points.len(), 4);
        assert_eq!(loaded.polys.num_cells(), 1);
    }

    #[test]
    fn comments_and_blank_lines() {
        let data =
            b"# This is a comment\nOFF\n# another comment\n\n3 1 0\n0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n";
        let mesh = OffReader::new(&data[..]).read().unwrap();
        assert_eq!(mesh.points.len(), 3);
    }
}
