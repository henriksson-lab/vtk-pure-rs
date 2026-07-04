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

        let (_, section) =
            crate::io::xml::vtp_reader::extract_section_with_tag(&content, "vtkMultiBlockDataSet")
                .ok_or_else(|| VtkError::Parse("missing vtkMultiBlockDataSet element".into()))?;

        parse_multiblock_section(&section, dir)
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

fn parse_multiblock_section(content: &str, dir: &Path) -> Result<MultiBlockDataSet, VtkError> {
    let mut mbd = MultiBlockDataSet::new();
    let mut search_pos = 0;

    while let Some((kind, start)) = next_child_element(content, search_pos) {
        match kind {
            "DataSet" => {
                let tag_end = content[start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed DataSet tag".into()))?;
                let tag = &content[start..start + tag_end + 1];
                let index = extract_attr(tag, "index")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_else(|| mbd.num_blocks());
                let name = extract_attr(tag, "name");
                let file = extract_attr(tag, "file");

                if let Some(ref filename) = file {
                    let block_path = dir.join(filename);
                    if let Some(block) = load_block(&block_path, filename) {
                        mbd.set_block(index, block);
                        if let Some(name) = name {
                            mbd.set_block_name(index, Some(name));
                        }
                    }
                }
                search_pos = start + tag_end + 1;
            }
            "Block" => {
                let tag_end = content[start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed Block tag".into()))?;
                let tag = &content[start..start + tag_end + 1];
                let index = extract_attr(tag, "index")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_else(|| mbd.num_blocks());
                let name = extract_attr(tag, "name");
                if tag.trim_end().ends_with("/>") {
                    mbd.set_block(index, Block::MultiBlock(MultiBlockDataSet::new()));
                    if let Some(name) = name {
                        mbd.set_block_name(index, Some(name));
                    }
                    search_pos = start + tag_end + 1;
                    continue;
                }
                let body_start = start + tag_end + 1;
                let close_start = find_matching_block_close(content, body_start)?;
                let child = parse_multiblock_section(&content[body_start..close_start], dir)?;
                mbd.set_block(index, Block::MultiBlock(child));
                if let Some(name) = name {
                    mbd.set_block_name(index, Some(name));
                }
                search_pos = close_start + "</Block>".len();
            }
            _ => unreachable!(),
        }
    }

    Ok(mbd)
}

fn next_child_element(content: &str, search_pos: usize) -> Option<(&'static str, usize)> {
    let ds = content[search_pos..]
        .find("<DataSet")
        .map(|p| search_pos + p);
    let block = content[search_pos..].find("<Block").map(|p| search_pos + p);
    match (ds, block) {
        (Some(ds), Some(block)) if ds < block => Some(("DataSet", ds)),
        (Some(_), Some(block)) => Some(("Block", block)),
        (Some(ds), None) => Some(("DataSet", ds)),
        (None, Some(block)) => Some(("Block", block)),
        (None, None) => None,
    }
}

fn find_matching_block_close(content: &str, mut search_pos: usize) -> Result<usize, VtkError> {
    let mut depth = 1usize;
    loop {
        let next_open = content[search_pos..].find("<Block").map(|p| search_pos + p);
        let next_close = content[search_pos..]
            .find("</Block>")
            .map(|p| search_pos + p);
        match (next_open, next_close) {
            (_, Some(close)) if next_open.map(|open| close < open).unwrap_or(true) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(close);
                }
                search_pos = close + "</Block>".len();
            }
            (Some(open), Some(_)) => {
                depth += 1;
                let tag_end = content[open..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed Block tag".into()))?;
                search_pos = open + tag_end + 1;
            }
            _ => return Err(VtkError::Parse("missing </Block>".into())),
        }
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
        assert!(xml.contains("file=\"data/data_0.vtp\""));
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
        std::fs::create_dir_all(dir.join("data")).unwrap();
        VtpWriter::write(&dir.join("data/data_0.vtp"), &pd).unwrap();

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
