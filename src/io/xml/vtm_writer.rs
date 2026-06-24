use std::io::Write;
use std::path::Path;

use crate::data::{Block, MultiBlockDataSet};
use crate::types::VtkError;

/// Writer for VTK XML MultiBlock format (.vtm).
///
/// Writes a `.vtm` file that references individual dataset files.
/// The individual files are written alongside the `.vtm` file.
pub struct VtmWriter;

impl VtmWriter {
    /// Write a MultiBlockDataSet to a directory.
    /// Creates `path` as the .vtm file and sibling files for each block.
    pub fn write(path: &Path, data: &MultiBlockDataSet) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_index_to(&mut w, data)
    }

    /// Write just the .vtm index (no data files). Useful for testing.
    pub fn write_index_to<W: Write>(w: &mut W, data: &MultiBlockDataSet) -> Result<(), VtkError> {
        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(w, "<VTKFile type=\"vtkMultiBlockDataSet\" version=\"1.0\">")?;
        writeln!(w, "  <vtkMultiBlockDataSet>")?;

        for (i, (name, block)) in data.iter().enumerate() {
            let block_name = name.unwrap_or("block");
            let type_str = match block {
                Block::PolyData(_) => "vtp",
                Block::ImageData(_) => "vti",
                Block::UnstructuredGrid(_) => "vtu",
                Block::RectilinearGrid(_) => "vtr",
                Block::StructuredGrid(_) => "vts",
                Block::MultiBlock(_) => "vtm",
            };
            writeln!(
                w,
                "    <DataSet index=\"{}\" name=\"{}\" file=\"{}_{}.{}\"/>",
                i, block_name, block_name, i, type_str
            )?;
        }

        writeln!(w, "  </vtkMultiBlockDataSet>")?;
        writeln!(w, "</VTKFile>")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{ImageData, PolyData};

    #[test]
    fn write_vtm_index() {
        let mut mb = MultiBlockDataSet::new();
        mb.add_block("mesh", Block::PolyData(PolyData::new()));
        mb.add_block(
            "volume",
            Block::ImageData(ImageData::with_dimensions(2, 2, 2)),
        );

        let mut buf = Vec::new();
        VtmWriter::write_index_to(&mut buf, &mb).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<VTKFile type=\"vtkMultiBlockDataSet\""));
        assert!(output.contains("name=\"mesh\""));
        assert!(output.contains("file=\"mesh_0.vtp\""));
        assert!(output.contains("name=\"volume\""));
        assert!(output.contains("file=\"volume_1.vti\""));
    }
}
