use std::io::Write;
use std::path::Path;

use crate::data::PolyData;
use crate::types::VtkError;

/// Writer for Stanford PLY format (binary little-endian).
pub struct PlyBinaryWriter;

impl PlyBinaryWriter {
    pub fn write(path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        let n_verts = data.points.len();
        let n_faces = data.polys.num_cells();

        // Header (always ASCII)
        writeln!(w, "ply")?;
        writeln!(w, "format binary_little_endian 1.0")?;
        writeln!(w, "comment vtk-rs export")?;
        writeln!(w, "element vertex {}", n_verts)?;
        writeln!(w, "property float x")?;
        writeln!(w, "property float y")?;
        writeln!(w, "property float z")?;
        writeln!(w, "element face {}", n_faces)?;
        writeln!(w, "property list uchar int vertex_indices")?;
        writeln!(w, "end_header")?;

        // Vertices (binary f32 little-endian)
        for i in 0..n_verts {
            let p = data.points.get(i);
            w.write_all(&(p[0] as f32).to_le_bytes())?;
            w.write_all(&(p[1] as f32).to_le_bytes())?;
            w.write_all(&(p[2] as f32).to_le_bytes())?;
        }

        // Faces (binary)
        for cell in data.polys.iter() {
            w.write_all(&[cell.len() as u8])?;
            for &id in cell {
                w.write_all(&(id as i32).to_le_bytes())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_binary_ply() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        // Header should be ASCII
        let header_end = buf.windows(11).position(|w| w == b"end_header\n").unwrap() + 11;
        let header = std::str::from_utf8(&buf[..header_end]).unwrap();
        assert!(header.contains("binary_little_endian"));
        // Binary data: 3 vertices * 12 bytes + 1 face * (1 + 12) bytes
        assert_eq!(buf.len() - header_end, 3 * 12 + 1 * 13);
    }
}
