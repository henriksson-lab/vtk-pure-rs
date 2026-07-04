use crate::data::{CellArray, Points, PolyData};
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
        let mut line_buf = String::new();

        if self
            .reader
            .read_line(&mut line_buf)
            .map_err(|e| e.to_string())?
            == 0
        {
            return Err("empty OFF file".into());
        }

        let header = line_buf.trim_end_matches('\n').trim_end_matches('\r');
        if header != "OFF" {
            return Err(format!("not an OFF file, got: {header}"));
        }

        // Parse counts: nVertices nFaces nEdges
        let counts_line = self.next_data_line("counts")?;
        let mut counts = counts_line.split_whitespace();
        let n_verts: usize = parse_next(&mut counts, "number of points")?;
        let n_faces: usize = parse_next(&mut counts, "number of polygons")?;
        if n_verts == 0 {
            return Err("file contains 0 points".into());
        }
        if n_faces == 0 {
            return Err("file contains 0 polygons".into());
        }

        // Parse vertices
        let mut points = Points::<f64>::new();

        for _ in 0..n_verts {
            let point_line = self.next_data_line("point coordinates")?;
            let mut parts = point_line.split_whitespace();
            let x: f64 = parse_next(&mut parts, "point x")?;
            let y: f64 = parse_next(&mut parts, "point y")?;
            let z: f64 = parse_next(&mut parts, "point z")?;
            points.push([x, y, z]);
        }

        // Parse faces
        let mut polys = CellArray::new();
        for i in 0..n_faces {
            let face_line = self.next_data_line("face")?;
            let mut parts = face_line.split_whitespace();
            let n: usize = parse_next(&mut parts, "face point count")?;
            if n < 1 {
                return Err(format!("face {i}: expected at least 1 index"));
            }
            if n > 100 {
                return Err(format!(
                    "face {i}: point count exceeds maximum allowed count of 100"
                ));
            }

            let mut ids = Vec::with_capacity(n);
            for j in 0..n {
                let id: i64 = parse_next(&mut parts, &format!("face {i} point index {j}"))?;
                ids.push(id);
            }
            for &id in &ids {
                if id < 0 || id as usize >= n_verts {
                    return Err(format!("face {i}: invalid point index {id}"));
                }
            }
            polys.push_cell(&ids);
        }

        let mut mesh = PolyData::new();
        mesh.points = points;
        mesh.polys = polys;

        Ok(mesh)
    }

    fn next_data_line(&mut self, context: &str) -> Result<String, String> {
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            if self
                .reader
                .read_line(&mut line_buf)
                .map_err(|e| e.to_string())?
                == 0
            {
                return Err(format!(
                    "unexpected end of OFF file while reading {context}"
                ));
            }
            let trimmed = line_buf.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            return Ok(trimmed.to_string());
        }
    }
}

fn parse_next<T: std::str::FromStr>(
    tokens: &mut std::str::SplitWhitespace<'_>,
    context: &str,
) -> Result<T, String> {
    tokens
        .next()
        .ok_or_else(|| format!("failed to parse {context}"))?
        .parse()
        .map_err(|_| format!("failed to parse {context}"))
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
        assert!(reader.read().is_err());
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
        assert!(OffReader::new(&data[..]).read().is_err());
    }

    #[test]
    fn comments_and_blank_lines_after_header() {
        let data = b"OFF\n# another comment\n\n3 1 0\n0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n";
        let mesh = OffReader::new(&data[..]).read().unwrap();
        assert_eq!(mesh.points.len(), 3);
    }
}
