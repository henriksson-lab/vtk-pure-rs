use std::io::Write;
use std::path::Path;

use crate::data::PolyData;
use crate::types::VtkError;

/// Writer for Stanford PLY format (ASCII).
pub struct PlyWriter;

impl PlyWriter {
    pub fn write(path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        let n_verts = data.points.len();
        let n_faces = data.polys.num_cells();

        // Check for vertex colors (RGB as 3-component scalar)
        let has_colors = data
            .point_data()
            .scalars()
            .is_some_and(|s| s.num_components() == 3);

        // Header
        writeln!(w, "ply")?;
        writeln!(w, "format ascii 1.0")?;
        writeln!(w, "comment vtk-rs export")?;
        writeln!(w, "element vertex {}", n_verts)?;
        writeln!(w, "property float x")?;
        writeln!(w, "property float y")?;
        writeln!(w, "property float z")?;

        if has_colors {
            writeln!(w, "property uchar red")?;
            writeln!(w, "property uchar green")?;
            writeln!(w, "property uchar blue")?;
        }

        writeln!(w, "element face {}", n_faces)?;
        writeln!(w, "property list uchar int vertex_indices")?;
        writeln!(w, "end_header")?;

        // Vertices
        let scalars = if has_colors {
            data.point_data().scalars()
        } else {
            None
        };

        let mut cbuf = [0.0f64; 3];
        for i in 0..n_verts {
            let p = data.points.get(i);
            if let Some(s) = scalars {
                s.tuple_as_f64(i, &mut cbuf);
                writeln!(
                    w,
                    "{} {} {} {} {} {}",
                    p[0],
                    p[1],
                    p[2],
                    (cbuf[0].clamp(0.0, 1.0) * 255.0) as u8,
                    (cbuf[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (cbuf[2].clamp(0.0, 1.0) * 255.0) as u8,
                )?;
            } else {
                writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
            }
        }

        // Faces
        for cell in data.polys.iter() {
            write!(w, "{}", cell.len())?;
            for &id in cell {
                write!(w, " {}", id)?;
            }
            writeln!(w)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        PlyWriter::write_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.starts_with("ply\n"));
        assert!(output.contains("element vertex 3"));
        assert!(output.contains("element face 1"));
        assert!(output.contains("3 0 1 2"));
    }
}
