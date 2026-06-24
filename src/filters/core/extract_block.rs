//! Extract blocks from MultiBlockDataSet.
//!
//! Provides utilities to extract individual blocks by index, name, or type
//! from a MultiBlockDataSet.

use crate::data::{Block, ImageData, MultiBlockDataSet, PolyData, UnstructuredGrid};

/// Extract a block by index, returning a clone of the block.
pub fn extract_block(mb: &MultiBlockDataSet, index: usize) -> Option<Block> {
    mb.block(index).cloned()
}

/// Extract a block by name.
pub fn extract_block_by_name(mb: &MultiBlockDataSet, name: &str) -> Option<Block> {
    for i in 0..mb.num_blocks() {
        if mb.block_name(i) == Some(name) {
            return mb.block(i).cloned();
        }
    }
    None
}

/// Extract all PolyData blocks from a MultiBlockDataSet.
pub fn extract_poly_data_blocks(mb: &MultiBlockDataSet) -> Vec<PolyData> {
    let mut result = Vec::new();
    for i in 0..mb.num_blocks() {
        if let Some(Block::PolyData(pd)) = mb.block(i) {
            result.push(pd.clone());
        }
    }
    result
}

/// Extract all ImageData blocks from a MultiBlockDataSet.
pub fn extract_image_data_blocks(mb: &MultiBlockDataSet) -> Vec<ImageData> {
    let mut result = Vec::new();
    for i in 0..mb.num_blocks() {
        if let Some(Block::ImageData(id)) = mb.block(i) {
            result.push(id.clone());
        }
    }
    result
}

/// Extract all UnstructuredGrid blocks from a MultiBlockDataSet.
pub fn extract_unstructured_grid_blocks(mb: &MultiBlockDataSet) -> Vec<UnstructuredGrid> {
    let mut result = Vec::new();
    for i in 0..mb.num_blocks() {
        if let Some(Block::UnstructuredGrid(ug)) = mb.block(i) {
            result.push(ug.clone());
        }
    }
    result
}

/// Merge all PolyData blocks into a single PolyData.
pub fn merge_poly_data_blocks(mb: &MultiBlockDataSet) -> PolyData {
    let blocks = extract_poly_data_blocks(mb);
    if blocks.is_empty() {
        return PolyData::new();
    }
    if blocks.len() == 1 {
        return blocks.into_iter().next().unwrap();
    }

    let refs: Vec<&PolyData> = blocks.iter().collect();
    crate::filters::core::append::append(&refs)
}

/// Flatten nested MultiBlockDataSets into a single-level MultiBlockDataSet.
pub fn flatten_multi_block(mb: &MultiBlockDataSet) -> MultiBlockDataSet {
    let mut result = MultiBlockDataSet::new();
    flatten_recursive(mb, &mut result, "");
    result
}

fn flatten_recursive(mb: &MultiBlockDataSet, result: &mut MultiBlockDataSet, prefix: &str) {
    for i in 0..mb.num_blocks() {
        let name = mb.block_name(i).map(|n| {
            if prefix.is_empty() {
                n.to_string()
            } else {
                format!("{prefix}/{n}")
            }
        });

        match mb.block(i) {
            Some(Block::MultiBlock(sub)) => {
                let sub_prefix = name.as_deref().unwrap_or(prefix);
                flatten_recursive(sub, result, sub_prefix);
            }
            Some(block) => {
                if let Some(n) = name {
                    result.add_block(n, block.clone());
                } else {
                    result.add_unnamed_block(block.clone());
                }
            }
            None => {}
        }
    }
}

/// Get a summary of block types in a MultiBlockDataSet.
pub fn multi_block_summary(mb: &MultiBlockDataSet) -> String {
    let mut poly = 0;
    let mut image = 0;
    let mut unstruct = 0;
    let mut recti = 0;
    let mut struc = 0;
    let mut nested = 0;

    for i in 0..mb.num_blocks() {
        match mb.block(i) {
            Some(Block::PolyData(_)) => poly += 1,
            Some(Block::ImageData(_)) => image += 1,
            Some(Block::UnstructuredGrid(_)) => unstruct += 1,
            Some(Block::RectilinearGrid(_)) => recti += 1,
            Some(Block::StructuredGrid(_)) => struc += 1,
            Some(Block::MultiBlock(_)) => nested += 1,
            None => {}
        }
    }

    let mut parts = Vec::new();
    if poly > 0 {
        parts.push(format!("{poly} PolyData"));
    }
    if image > 0 {
        parts.push(format!("{image} ImageData"));
    }
    if unstruct > 0 {
        parts.push(format!("{unstruct} UnstructuredGrid"));
    }
    if recti > 0 {
        parts.push(format!("{recti} RectilinearGrid"));
    }
    if struc > 0 {
        parts.push(format!("{struc} StructuredGrid"));
    }
    if nested > 0 {
        parts.push(format!("{nested} nested MultiBlock"));
    }

    format!(
        "MultiBlock: {} blocks [{}]",
        mb.num_blocks(),
        parts.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_multi_block() -> MultiBlockDataSet {
        let mut mb = MultiBlockDataSet::new();
        mb.add_block(
            "sphere",
            Block::PolyData(PolyData::from_triangles(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                vec![[0, 1, 2]],
            )),
        );
        mb.add_block(
            "cube",
            Block::PolyData(PolyData::from_triangles(
                vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
                vec![[0, 1, 2]],
            )),
        );
        mb.add_block(
            "grid",
            Block::ImageData(ImageData::with_dimensions(5, 5, 5)),
        );
        mb
    }

    #[test]
    fn extract_by_index() {
        let mb = make_multi_block();
        let block = extract_block(&mb, 0).unwrap();
        assert!(matches!(block, Block::PolyData(_)));
    }

    #[test]
    fn extract_by_name() {
        let mb = make_multi_block();
        let block = extract_block_by_name(&mb, "cube").unwrap();
        assert!(matches!(block, Block::PolyData(_)));
        assert!(extract_block_by_name(&mb, "missing").is_none());
    }

    #[test]
    fn extract_typed_blocks() {
        let mb = make_multi_block();
        let polys = extract_poly_data_blocks(&mb);
        assert_eq!(polys.len(), 2);
        let images = extract_image_data_blocks(&mb);
        assert_eq!(images.len(), 1);
    }

    #[test]
    fn merge_blocks() {
        let mb = make_multi_block();
        let merged = merge_poly_data_blocks(&mb);
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);
    }

    #[test]
    fn flatten_nested() {
        let inner = make_multi_block();
        let mut outer = MultiBlockDataSet::new();
        outer.add_block("inner", Block::MultiBlock(inner));
        outer.add_block("extra", Block::PolyData(PolyData::new()));

        let flat = flatten_multi_block(&outer);
        assert_eq!(flat.num_blocks(), 4); // 3 from inner + 1 extra
    }

    #[test]
    fn summary() {
        let mb = make_multi_block();
        let s = multi_block_summary(&mb);
        assert!(s.contains("2 PolyData"));
        assert!(s.contains("1 ImageData"));
    }
}
