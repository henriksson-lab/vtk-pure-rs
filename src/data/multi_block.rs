use crate::data::traits::DataObject;
use crate::data::{
    FieldData, ImageData, PolyData, RectilinearGrid, StructuredGrid, UnstructuredGrid,
};

/// A block in a MultiBlockDataSet, which can hold different dataset types.
#[derive(Debug, Clone)]
pub enum Block {
    PolyData(PolyData),
    ImageData(ImageData),
    UnstructuredGrid(UnstructuredGrid),
    RectilinearGrid(RectilinearGrid),
    StructuredGrid(StructuredGrid),
    MultiBlock(MultiBlockDataSet),
}

impl From<PolyData> for Block {
    fn from(pd: PolyData) -> Self {
        Block::PolyData(pd)
    }
}
impl From<ImageData> for Block {
    fn from(id: ImageData) -> Self {
        Block::ImageData(id)
    }
}
impl From<UnstructuredGrid> for Block {
    fn from(ug: UnstructuredGrid) -> Self {
        Block::UnstructuredGrid(ug)
    }
}
impl From<RectilinearGrid> for Block {
    fn from(rg: RectilinearGrid) -> Self {
        Block::RectilinearGrid(rg)
    }
}
impl From<StructuredGrid> for Block {
    fn from(sg: StructuredGrid) -> Self {
        Block::StructuredGrid(sg)
    }
}
impl From<MultiBlockDataSet> for Block {
    fn from(mb: MultiBlockDataSet) -> Self {
        Block::MultiBlock(mb)
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::PolyData(pd) => write!(f, "{pd}"),
            Block::ImageData(id) => write!(f, "{id}"),
            Block::UnstructuredGrid(ug) => write!(f, "{ug}"),
            Block::RectilinearGrid(_) => write!(f, "RectilinearGrid"),
            Block::StructuredGrid(_) => write!(f, "StructuredGrid"),
            Block::MultiBlock(mb) => write!(f, "MultiBlock({} blocks)", mb.num_blocks()),
        }
    }
}

impl std::fmt::Display for MultiBlockDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultiBlockDataSet: {} blocks", self.num_blocks())
    }
}

/// A composite dataset containing named blocks of heterogeneous data types.
///
/// Analogous to VTK's `vtkMultiBlockDataSet`. Each block can be any dataset
/// type, including nested MultiBlockDataSets.
#[derive(Debug, Clone, Default)]
pub struct MultiBlockDataSet {
    blocks: Vec<(Option<String>, Option<Block>)>,
    field_data: FieldData,
}

impl MultiBlockDataSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named block.
    pub fn add_block(&mut self, name: impl Into<String>, block: Block) {
        self.blocks.push((Some(name.into()), Some(block)));
    }

    /// Add an unnamed block.
    pub fn add_unnamed_block(&mut self, block: Block) {
        self.blocks.push((None, Some(block)));
    }

    /// Set the number of blocks, initializing new slots to null.
    pub fn set_number_of_blocks(&mut self, num_blocks: usize) {
        self.blocks.resize_with(num_blocks, || (None, None));
    }

    /// Set a block at an index, growing with null slots as VTK's `SetBlock` does.
    pub fn set_block(&mut self, idx: usize, block: Block) -> Option<Block> {
        if idx >= self.blocks.len() {
            self.set_number_of_blocks(idx + 1);
        }
        let (_, existing) = &mut self.blocks[idx];
        existing.replace(block)
    }

    /// Clear a block while preserving its metadata/name slot.
    pub fn clear_block(&mut self, idx: usize) -> Option<Block> {
        self.blocks.get_mut(idx).and_then(|(_, block)| block.take())
    }

    /// Set or clear the name for an existing block.
    pub fn set_block_name(&mut self, idx: usize, name: Option<impl Into<String>>) -> bool {
        let Some((existing, _)) = self.blocks.get_mut(idx) else {
            return false;
        };
        *existing = name.map(Into::into);
        true
    }

    /// Number of blocks.
    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Get block by index.
    pub fn block(&self, idx: usize) -> Option<&Block> {
        self.blocks.get(idx).and_then(|(_, b)| b.as_ref())
    }

    /// Get block name by index.
    pub fn block_name(&self, idx: usize) -> Option<&str> {
        self.blocks.get(idx).and_then(|(n, _)| n.as_deref())
    }

    /// Get block by name (first match).
    pub fn block_by_name(&self, name: &str) -> Option<&Block> {
        self.blocks.iter().find_map(|(n, b)| {
            if n.as_deref() == Some(name) {
                b.as_ref()
            } else {
                None
            }
        })
    }

    /// Iterate over all blocks with their optional names.
    pub fn iter(&self) -> impl Iterator<Item = (Option<&str>, &Block)> {
        self.blocks
            .iter()
            .filter_map(|(n, b)| b.as_ref().map(|b| (n.as_deref(), b)))
    }

    /// Remove a block by index. Returns the removed block.
    pub fn remove_block(&mut self, idx: usize) -> (Option<String>, Block) {
        let (name, block) = self.blocks.remove(idx);
        (
            name,
            block.expect("cannot remove a null block as a concrete Block"),
        )
    }

    /// Remove a block by index, returning `None` if the index is out of bounds.
    pub fn try_remove_block(&mut self, idx: usize) -> Option<(Option<String>, Block)> {
        if idx >= self.blocks.len() {
            return None;
        }
        let (name, block) = self.blocks.remove(idx);
        block.map(|block| (name, block))
    }

    /// Remove a block by name. Returns the removed block if found.
    pub fn remove_by_name(&mut self, name: &str) -> Option<Block> {
        if let Some(idx) = self
            .blocks
            .iter()
            .position(|(n, b)| n.as_deref() == Some(name) && b.is_some())
        {
            self.blocks.remove(idx).1
        } else {
            None
        }
    }

    /// Get all PolyData blocks.
    pub fn poly_data_blocks(&self) -> Vec<(Option<&str>, &PolyData)> {
        self.blocks
            .iter()
            .filter_map(|(n, b)| match b.as_ref()? {
                Block::PolyData(pd) => Some((n.as_deref(), pd)),
                _ => None,
            })
            .collect()
    }

    /// Get all ImageData blocks.
    pub fn image_data_blocks(&self) -> Vec<(Option<&str>, &ImageData)> {
        self.blocks
            .iter()
            .filter_map(|(n, b)| match b.as_ref()? {
                Block::ImageData(id) => Some((n.as_deref(), id)),
                _ => None,
            })
            .collect()
    }

    /// Get all UnstructuredGrid blocks.
    pub fn unstructured_grid_blocks(&self) -> Vec<(Option<&str>, &UnstructuredGrid)> {
        self.blocks
            .iter()
            .filter_map(|(n, b)| match b.as_ref()? {
                Block::UnstructuredGrid(ug) => Some((n.as_deref(), ug)),
                _ => None,
            })
            .collect()
    }

    /// Flatten all nested MultiBlockDataSets into a single level.
    pub fn flatten(&self) -> Vec<(Option<String>, Block)> {
        let mut result = Vec::new();
        for (name, block) in &self.blocks {
            let Some(block) = block else {
                continue;
            };
            match block {
                Block::MultiBlock(inner) => {
                    for (sub_name, sub_block) in inner.flatten() {
                        let combined_name = match (name.as_deref(), sub_name.as_deref()) {
                            (Some(p), Some(c)) => Some(format!("{p}/{c}")),
                            (Some(p), None) => Some(p.to_string()),
                            (None, Some(c)) => Some(c.to_string()),
                            (None, None) => None,
                        };
                        result.push((combined_name, sub_block));
                    }
                }
                other => {
                    result.push((name.clone(), other.clone()));
                }
            }
        }
        result
    }

    /// Builder: add a named block.
    pub fn with_block(mut self, name: impl Into<String>, block: Block) -> Self {
        self.add_block(name, block);
        self
    }

    /// Total number of blocks at all levels (recursive).
    pub fn total_blocks_recursive(&self) -> usize {
        let mut count = 0;
        for (_, block) in &self.blocks {
            match block {
                None => {}
                Some(block) => match block {
                    Block::MultiBlock(inner) => count += inner.total_blocks_recursive(),
                    _ => count += 1,
                },
            }
        }
        count
    }
}

impl DataObject for MultiBlockDataSet {
    fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_block_basics() {
        let mut mb = MultiBlockDataSet::new();
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mb.add_block("mesh", Block::PolyData(pd));
        mb.add_block(
            "image",
            Block::ImageData(ImageData::with_dimensions(2, 2, 2)),
        );

        assert_eq!(mb.num_blocks(), 2);
        assert_eq!(mb.block_name(0), Some("mesh"));
        assert!(mb.block_by_name("image").is_some());
    }

    #[test]
    fn typed_getters() {
        let mb = MultiBlockDataSet::new()
            .with_block("tri", Block::PolyData(PolyData::new()))
            .with_block("img", Block::ImageData(ImageData::with_dimensions(2, 2, 2)));
        assert_eq!(mb.poly_data_blocks().len(), 1);
        assert_eq!(mb.image_data_blocks().len(), 1);
        assert_eq!(mb.unstructured_grid_blocks().len(), 0);
    }

    #[test]
    fn remove_by_name() {
        let mut mb = MultiBlockDataSet::new();
        mb.add_block("a", Block::PolyData(PolyData::new()));
        mb.add_block("b", Block::PolyData(PolyData::new()));
        assert_eq!(mb.num_blocks(), 2);
        mb.remove_by_name("a");
        assert_eq!(mb.num_blocks(), 1);
        assert_eq!(mb.block_name(0), Some("b"));
    }

    #[test]
    fn flatten() {
        let inner = MultiBlockDataSet::new()
            .with_block("c1", Block::PolyData(PolyData::new()))
            .with_block("c2", Block::PolyData(PolyData::new()));
        let outer = MultiBlockDataSet::new()
            .with_block("top", Block::PolyData(PolyData::new()))
            .with_block("sub", Block::MultiBlock(inner));
        let flat = outer.flatten();
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].0.as_deref(), Some("top"));
        assert_eq!(flat[1].0.as_deref(), Some("sub/c1"));
    }

    #[test]
    fn total_recursive() {
        let inner = MultiBlockDataSet::new()
            .with_block("a", Block::PolyData(PolyData::new()))
            .with_block("b", Block::PolyData(PolyData::new()));
        let outer = MultiBlockDataSet::new()
            .with_block("top", Block::PolyData(PolyData::new()))
            .with_block("sub", Block::MultiBlock(inner));
        assert_eq!(outer.total_blocks_recursive(), 3);
    }

    #[test]
    fn nested_multi_block() {
        let mut inner = MultiBlockDataSet::new();
        inner.add_block("tri", Block::PolyData(PolyData::new()));

        let mut outer = MultiBlockDataSet::new();
        outer.add_block("sub", Block::MultiBlock(inner));

        assert_eq!(outer.num_blocks(), 1);
        if let Some(Block::MultiBlock(sub)) = outer.block(0) {
            assert_eq!(sub.num_blocks(), 1);
        } else {
            panic!("expected nested MultiBlock");
        }
    }

    #[test]
    fn from_conversions() {
        let pd = PolyData::new();
        let block: Block = pd.into();
        assert!(matches!(block, Block::PolyData(_)));

        let img = ImageData::with_dimensions(2, 2, 2);
        let block: Block = img.into();
        assert!(matches!(block, Block::ImageData(_)));
    }

    #[test]
    fn display() {
        let mb = MultiBlockDataSet::new()
            .with_block("tri", PolyData::new().into())
            .with_block("img", ImageData::with_dimensions(2, 2, 2).into());
        let s = format!("{mb}");
        assert!(s.contains("2 blocks"));

        let block: Block = PolyData::new().into();
        let s = format!("{block}");
        assert!(s.contains("PolyData"));
    }

    #[test]
    fn checked_mutation_and_removal() {
        let mut mb = MultiBlockDataSet::new();
        mb.add_block("empty", PolyData::new().into());
        assert_eq!(mb.num_blocks(), 1);
        assert!(mb.set_block_name(0, Some("renamed")));
        assert_eq!(mb.block_name(0), Some("renamed"));
        assert!(matches!(
            mb.set_block(0, ImageData::with_dimensions(1, 1, 1).into()),
            Some(Block::PolyData(_))
        ));
        assert!(mb.set_block(3, PolyData::new().into()).is_none());
        assert_eq!(mb.num_blocks(), 4);
        assert!(mb.block(1).is_none());
        assert!(mb.try_remove_block(10).is_none());
        assert!(mb.try_remove_block(0).is_some());
        assert_eq!(mb.num_blocks(), 3);
    }

    #[test]
    fn set_number_of_blocks_creates_null_slots() {
        let mut mb = MultiBlockDataSet::new();
        mb.set_number_of_blocks(2);
        assert_eq!(mb.num_blocks(), 2);
        assert!(mb.block(0).is_none());
        mb.set_block_name(1, Some("empty"));
        assert_eq!(mb.block_name(1), Some("empty"));
    }
}
