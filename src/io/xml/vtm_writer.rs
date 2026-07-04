use std::io::Write;
use std::path::Path;

use crate::data::{Block, MultiBlockDataSet};
use crate::types::VtkError;

/// Writer for VTK XML MultiBlock format (.vtm).
///
/// Writes a `.vtm` file that references individual dataset files.
/// The individual files are written under a stem-named subdirectory.
pub struct VtmWriter;

impl VtmWriter {
    /// Write a MultiBlockDataSet to a directory.
    /// Creates `path` as the .vtm file and sidecar files for each block.
    pub fn write(path: &Path, data: &MultiBlockDataSet) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        let file_prefix = piece_file_prefix(path);
        Self::write_index_to_with_prefix(&mut w, data, &file_prefix)?;

        let dir = path.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir.join(&file_prefix))?;
        write_block_files(dir, data, &file_prefix)
    }

    /// Write just the .vtm index (no data files). Useful for testing.
    pub fn write_index_to<W: Write>(w: &mut W, data: &MultiBlockDataSet) -> Result<(), VtkError> {
        Self::write_index_to_with_prefix(w, data, "data")
    }

    fn write_index_to_with_prefix<W: Write>(
        w: &mut W,
        data: &MultiBlockDataSet,
        file_prefix: &str,
    ) -> Result<(), VtkError> {
        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(w, "<VTKFile type=\"vtkMultiBlockDataSet\" version=\"1.0\">")?;
        writeln!(w, "  <vtkMultiBlockDataSet>")?;
        let mut writer_idx = 0;
        write_blocks_xml(w, data, 4, &mut writer_idx, file_prefix)?;
        writeln!(w, "  </vtkMultiBlockDataSet>")?;
        writeln!(w, "</VTKFile>")?;
        Ok(())
    }
}

fn write_blocks_xml<W: Write>(
    w: &mut W,
    data: &MultiBlockDataSet,
    indent: usize,
    writer_idx: &mut usize,
    file_prefix: &str,
) -> Result<(), VtkError> {
    let pad = " ".repeat(indent);
    for (i, (name, block)) in data.iter().enumerate() {
        match block {
            Block::MultiBlock(mb) => {
                let name_attr = name
                    .map(|name| format!(" name=\"{}\"", xml_escape_attr(name)))
                    .unwrap_or_default();
                writeln!(w, "{pad}<Block index=\"{i}\"{name_attr}>")?;
                write_blocks_xml(w, mb, indent + 2, writer_idx, file_prefix)?;
                writeln!(w, "{pad}</Block>")?;
            }
            _ => {
                let ext = block_extension(block);
                let file = xml_escape_attr(&block_file_name(file_prefix, *writer_idx, ext));
                *writer_idx += 1;
                let name_attr = name
                    .map(|name| format!(" name=\"{}\"", xml_escape_attr(name)))
                    .unwrap_or_default();
                writeln!(
                    w,
                    "{pad}<DataSet index=\"{i}\"{name_attr} file=\"{file}\"/>"
                )?;
            }
        }
    }
    Ok(())
}

fn write_block_files(
    dir: &Path,
    data: &MultiBlockDataSet,
    file_prefix: &str,
) -> Result<(), VtkError> {
    let mut writer_idx = 0;
    write_block_files_impl(dir, data, &mut writer_idx, file_prefix)
}

fn write_block_files_impl(
    dir: &Path,
    data: &MultiBlockDataSet,
    writer_idx: &mut usize,
    file_prefix: &str,
) -> Result<(), VtkError> {
    for (_, block) in data.iter() {
        match block {
            Block::PolyData(pd) => {
                let file = block_file_name(file_prefix, *writer_idx, "vtp");
                *writer_idx += 1;
                crate::io::xml::VtpWriter::write(&dir.join(file), pd)?
            }
            Block::ImageData(img) => {
                let file = block_file_name(file_prefix, *writer_idx, "vti");
                *writer_idx += 1;
                crate::io::xml::VtiWriter::write(&dir.join(file), img)?
            }
            Block::UnstructuredGrid(grid) => {
                let file = block_file_name(file_prefix, *writer_idx, "vtu");
                *writer_idx += 1;
                crate::io::xml::VtuWriter::write(&dir.join(file), grid)?
            }
            Block::RectilinearGrid(grid) => {
                let file = block_file_name(file_prefix, *writer_idx, "vtr");
                *writer_idx += 1;
                crate::io::xml::VtrWriter::write(&dir.join(file), grid)?
            }
            Block::StructuredGrid(grid) => {
                let file = block_file_name(file_prefix, *writer_idx, "vts");
                *writer_idx += 1;
                crate::io::xml::VtsWriter::write(&dir.join(file), grid)?
            }
            Block::MultiBlock(mb) => write_block_files_impl(dir, mb, writer_idx, file_prefix)?,
        }
    }
    Ok(())
}

fn block_extension(block: &Block) -> &'static str {
    match block {
        Block::PolyData(_) => "vtp",
        Block::ImageData(_) => "vti",
        Block::UnstructuredGrid(_) => "vtu",
        Block::RectilinearGrid(_) => "vtr",
        Block::StructuredGrid(_) => "vts",
        Block::MultiBlock(_) => "vtm",
    }
}

fn block_file_name(block_name: &str, index: usize, ext: &str) -> String {
    format!("{}/{}_{}.{}", block_name, block_name, index, ext)
}

fn piece_file_prefix(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("data")
        .to_string()
}

fn xml_escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
        assert!(output.contains("file=\"data/data_0.vtp\""));
        assert!(output.contains("name=\"volume\""));
        assert!(output.contains("file=\"data/data_1.vti\""));
    }
}
