use crate::data::PolyData;
use std::io::Write;

/// Writer for Object File Format (OFF).
pub struct OffWriter<W: Write> {
    writer: W,
}

impl<W: Write> OffWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a PolyData mesh in OFF format.
    pub fn write(&mut self, mesh: &PolyData) -> std::io::Result<()> {
        let npts = mesh.points.len();

        // Collect faces (polys + strips as triangles)
        let faces = collect_faces(mesh);
        let nfaces = faces.len();

        // Count total edges (for the header; OFF header is: nVertices nFaces nEdges)
        // nEdges can be 0 (optional in OFF)
        let has_colors = mesh.point_data().scalars().is_some()
            && mesh.point_data().scalars().unwrap().num_components() >= 3;

        if has_colors {
            writeln!(self.writer, "COFF")?;
        } else {
            writeln!(self.writer, "OFF")?;
        }
        writeln!(self.writer, "{npts} {nfaces} 0")?;

        // Vertices
        let scalars = if has_colors {
            mesh.point_data().scalars()
        } else {
            None
        };

        for i in 0..npts {
            let p = mesh.points.get(i);
            if let Some(s) = scalars {
                let nc = s.num_components();
                let mut buf = vec![0.0f64; nc];
                s.tuple_as_f64(i, &mut buf);
                let r = (buf[0].clamp(0.0, 1.0) * 255.0) as u8;
                let g = (buf[1].clamp(0.0, 1.0) * 255.0) as u8;
                let b = (buf[2].clamp(0.0, 1.0) * 255.0) as u8;
                let a = if nc >= 4 {
                    (buf[3].clamp(0.0, 1.0) * 255.0) as u8
                } else {
                    255
                };
                writeln!(
                    self.writer,
                    "{} {} {} {} {} {} {}",
                    p[0], p[1], p[2], r, g, b, a
                )?;
            } else {
                writeln!(self.writer, "{} {} {}", p[0], p[1], p[2])?;
            }
        }

        // Faces
        for face in &faces {
            let mut line = format!("{}", face.len());
            for &idx in face {
                line.push_str(&format!(" {idx}"));
            }
            writeln!(self.writer, "{line}")?;
        }

        Ok(())
    }
}

fn collect_faces(mesh: &PolyData) -> Vec<Vec<usize>> {
    let mut faces = Vec::new();

    // Polygons
    for cell in mesh.polys.iter() {
        faces.push(cell.iter().map(|&id| id as usize).collect());
    }

    // Triangle strips → triangles
    for cell in mesh.strips.iter() {
        if cell.len() >= 3 {
            for i in 2..cell.len() {
                if i % 2 == 0 {
                    faces.push(vec![
                        cell[i - 2] as usize,
                        cell[i - 1] as usize,
                        cell[i] as usize,
                    ]);
                } else {
                    faces.push(vec![
                        cell[i - 1] as usize,
                        cell[i - 2] as usize,
                        cell[i] as usize,
                    ]);
                }
            }
        }
    }

    faces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        OffWriter::new(&mut buf).write(&mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("OFF\n"));
        assert!(s.contains("3 1 0"));
    }

    #[test]
    fn write_quad() {
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
        OffWriter::new(&mut buf).write(&mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("4 0 1 2 3"));
    }
}
