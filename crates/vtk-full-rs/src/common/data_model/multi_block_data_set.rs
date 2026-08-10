use super::DataObject;
use std::sync::Arc;

/// Shared storage for composite child block slots.
#[derive(Debug, Clone, Default, PartialEq)]
struct MultiBlockStorage {
    blocks: Vec<Option<DataObject>>,
}

/// Composite dataset containing ordered child blocks.
///
/// VTK origin: `VTK/Common/DataModel/vtkMultiBlockDataSet.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiBlockDataSet {
    storage: Arc<MultiBlockStorage>,
}

impl MultiBlockDataSet {
    /// VTK: `vtkMultiBlockDataSet::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(MultiBlockStorage::default()),
        }
    }

    /// VTK: `vtkMultiBlockDataSet::SetNumberOfBlocks`.
    pub fn set_number_of_blocks(&mut self, num_blocks: u32) {
        self.storage_mut()
            .blocks
            .resize_with(num_blocks as usize, || None);
    }

    /// VTK: `vtkMultiBlockDataSet::GetNumberOfBlocks`.
    pub fn get_number_of_blocks(&self) -> u32 {
        self.storage.blocks.len() as u32
    }

    /// VTK: `vtkMultiBlockDataSet::GetBlock`.
    pub fn get_block(&self, block_no: u32) -> Option<&DataObject> {
        self.storage
            .blocks
            .get(block_no as usize)
            .and_then(Option::as_ref)
    }

    /// VTK: `vtkMultiBlockDataSet::SetBlock`.
    pub fn set_block(&mut self, block_no: u32, block: Option<DataObject>) {
        if let Some(block) = block.as_ref() {
            if !Self::validate_block(block) {
                return;
            }
        }
        let block_no = block_no as usize;
        if block_no >= self.storage.blocks.len() {
            self.set_number_of_blocks((block_no + 1) as u32);
        }
        self.storage_mut().blocks[block_no] = block;
    }

    /// VTK: `vtkMultiBlockDataSet::RemoveBlock`.
    pub fn remove_block(&mut self, block_no: u32) {
        let storage = self.storage_mut();
        let block_no = block_no as usize;
        if block_no < storage.blocks.len() {
            storage.blocks.remove(block_no);
        }
    }

    /// VTK: `vtkMultiBlockDataSet::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkMultiBlockDataSet\n  Number Of Blocks: {}",
            self.get_number_of_blocks()
        )
    }

    fn validate_block(block: &DataObject) -> bool {
        !matches!(
            block.get_class_name(),
            "vtkUniformGridAMR" | "vtkPartitionedDataSet" | "vtkPartitionedDataSetCollection"
        )
    }

    /// VTK: `vtkMultiBlockDataSet::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.storage = Arc::new((*source.storage).clone());
    }

    /// VTK: `vtkMultiBlockDataSet::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Arc::clone(&source.storage);
    }

    fn storage_mut(&mut self) -> &mut MultiBlockStorage {
        Arc::make_mut(&mut self.storage)
    }
}
