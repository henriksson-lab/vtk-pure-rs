use crate::data::PolyData;
use std::io::Write;

/// Writer for AutoCAD DXF format.
pub struct DxfWriter<W: Write> {
    writer: W,
}

impl<W: Write> DxfWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a PolyData mesh as DXF with 3DFACE entities for polygons
    /// and LINE entities for line cells.
    pub fn write(&mut self, mesh: &PolyData) -> std::io::Result<()> {
        // Header section
        writeln!(self.writer, "  0")?;
        writeln!(self.writer, "SECTION")?;
        writeln!(self.writer, "  2")?;
        writeln!(self.writer, "HEADER")?;
        writeln!(self.writer, "  0")?;
        writeln!(self.writer, "ENDSEC")?;

        // Entities section
        writeln!(self.writer, "  0")?;
        writeln!(self.writer, "SECTION")?;
        writeln!(self.writer, "  2")?;
        writeln!(self.writer, "ENTITIES")?;

        // Write polygons as 3DFACE entities
        for cell in mesh.polys.iter() {
            if cell.len() >= 3 {
                let p0 = mesh.points.get(cell[0] as usize);
                let p1 = mesh.points.get(cell[1] as usize);
                let p2 = mesh.points.get(cell[2] as usize);
                let p3 = if cell.len() >= 4 {
                    mesh.points.get(cell[3] as usize)
                } else {
                    p2 // degenerate quad = triangle
                };

                writeln!(self.writer, "  0")?;
                writeln!(self.writer, "3DFACE")?;
                writeln!(self.writer, "  8")?;
                writeln!(self.writer, "0")?; // layer

                write_point(&mut self.writer, 10, p0)?;
                write_point(&mut self.writer, 11, p1)?;
                write_point(&mut self.writer, 12, p2)?;
                write_point(&mut self.writer, 13, p3)?;
            }
        }

        // Write line cells as LINE entities
        for cell in mesh.lines.iter() {
            for i in 0..cell.len().saturating_sub(1) {
                let p0 = mesh.points.get(cell[i] as usize);
                let p1 = mesh.points.get(cell[i + 1] as usize);

                writeln!(self.writer, "  0")?;
                writeln!(self.writer, "LINE")?;
                writeln!(self.writer, "  8")?;
                writeln!(self.writer, "0")?;

                write_point(&mut self.writer, 10, p0)?;
                write_point(&mut self.writer, 11, p1)?;
            }
        }

        writeln!(self.writer, "  0")?;
        writeln!(self.writer, "ENDSEC")?;
        writeln!(self.writer, "  0")?;
        writeln!(self.writer, "EOF")?;

        Ok(())
    }
}

fn write_point<W: Write>(w: &mut W, base_code: i32, p: [f64; 3]) -> std::io::Result<()> {
    writeln!(w, " {base_code}")?;
    writeln!(w, "{}", p[0])?;
    writeln!(w, " {}", base_code + 10)?;
    writeln!(w, "{}", p[1])?;
    writeln!(w, " {}", base_code + 20)?;
    writeln!(w, "{}", p[2])?;
    Ok(())
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
        DxfWriter::new(&mut buf).write(&mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("3DFACE"));
        assert!(s.contains("EOF"));
    }
}
