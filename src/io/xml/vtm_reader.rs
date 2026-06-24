use std::path::Path;

use crate::data::{Block, MultiBlockDataSet};
use crate::types::VtkError;

use crate::io::xml::vtp_reader::extract_attr;

/// Reader for VTK XML MultiBlock format (.vtm).
///
/// Reads the `.vtm` index file and loads referenced dataset files.
pub struct VtmReader;

impl VtmReader {
    /// Read a .vtm file and load all referenced blocks.
    pub fn read(path: &Path) -> Result<MultiBlockDataSet, VtkError> {
        let content = std::fs::read_to_string(path)?;
        let dir = path.parent().unwrap_or(Path::new("."));

        let mut mbd = MultiBlockDataSet::new();
        let mut search_pos = 0;

        while let Some(ds_start) = content[search_pos..].find("<DataSet") {
            let abs_start = search_pos + ds_start;
            let tag_end = content[abs_start..]
                .find("/>")
                .or_else(|| content[abs_start..].find('>'))
                .ok_or_else(|| VtkError::Parse("unclosed DataSet tag".into()))?;
            let tag = &content[abs_start..abs_start + tag_end + 2];

            let name = extract_attr(tag, "name");
            let file = extract_attr(tag, "file");

            if let Some(ref filename) = file {
                let block_path = dir.join(filename);
                if let Some(block) = load_block(&block_path, filename) {
                    mbd.add_block(name.unwrap_or_else(|| "block".to_string()), block);
                }
            }

            search_pos = abs_start + tag_end + 2;
        }

        Ok(mbd)
    }

    /// Read just the index (names and file references) without loading data.
    pub fn read_index(path: &Path) -> Result<Vec<(Option<String>, String)>, VtkError> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();
        let mut search_pos = 0;

        while let Some(ds_start) = content[search_pos..].find("<DataSet") {
            let abs_start = search_pos + ds_start;
            let tag_end = content[abs_start..]
                .find("/>")
                .or_else(|| content[abs_start..].find('>'))
                .ok_or_else(|| VtkError::Parse("unclosed DataSet tag".into()))?;
            let tag = &content[abs_start..abs_start + tag_end + 2];

            let name = extract_attr(tag, "name");
            let file = extract_attr(tag, "file").unwrap_or_default();
            entries.push((name, file));

            search_pos = abs_start + tag_end + 2;
        }

        Ok(entries)
    }
}

fn load_block(path: &Path, filename: &str) -> Option<Block> {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext {
        "vtp" => crate::io::xml::VtpReader::read(path)
            .ok()
            .map(Block::PolyData),
        "vtu" => crate::io::xml::VtuReader::read(path)
            .ok()
            .map(Block::UnstructuredGrid),
        "vti" => crate::io::xml::VtiReader::read(path)
            .ok()
            .map(Block::ImageData),
        "vtr" => crate::io::xml::VtrReader::read(path)
            .ok()
            .map(Block::RectilinearGrid),
        "vts" => crate::io::xml::VtsReader::read(path)
            .ok()
            .map(Block::StructuredGrid),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;
    use crate::io::xml::{VtmWriter, VtpWriter};

    #[test]
    fn read_vtm_index() {
        let mut mbd = MultiBlockDataSet::new();
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mbd.add_block("mesh", Block::PolyData(pd));

        let mut buf = Vec::new();
        VtmWriter::write_index_to(&mut buf, &mbd).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains("name=\"mesh\""));
        assert!(xml.contains("file=\"mesh_0.vtp\""));
    }

    #[test]
    fn roundtrip_vtm_with_files() {
        let dir = std::env::temp_dir().join("vtk_vtm_rt_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Write a VTP file
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        VtpWriter::write(&dir.join("mesh_0.vtp"), &pd).unwrap();

        // Write a VTM index referencing it
        let mut mbd = MultiBlockDataSet::new();
        mbd.add_block("mesh", Block::PolyData(pd));
        VtmWriter::write(&dir.join("data.vtm"), &mbd).unwrap();

        // Read back
        let result = VtmReader::read(&dir.join("data.vtm")).unwrap();
        assert_eq!(result.num_blocks(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
