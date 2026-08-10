use super::DataObject;
use std::sync::Arc;

/// Shared storage for partition slots.
#[derive(Debug, Clone, Default, PartialEq)]
struct PartitionedDataSetStorage {
    partitions: Vec<Option<DataObject>>,
}

/// Composite dataset containing leaf dataset partitions.
///
/// VTK origin: `VTK/Common/DataModel/vtkPartitionedDataSet.{h,cxx}`.
///
/// VTK stores arbitrary `vtkDataObject` children but rejects composite
/// datasets.
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedDataSet {
    storage: Arc<PartitionedDataSetStorage>,
}

impl PartitionedDataSet {
    /// VTK: `vtkPartitionedDataSet::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(PartitionedDataSetStorage::default()),
        }
    }

    /// VTK: `vtkPartitionedDataSet::SetNumberOfPartitions`.
    pub fn set_number_of_partitions(&mut self, num_partitions: u32) {
        self.storage_mut()
            .partitions
            .resize_with(num_partitions as usize, || None);
    }

    /// VTK: `vtkPartitionedDataSet::GetNumberOfPartitions`.
    pub fn get_number_of_partitions(&self) -> u32 {
        self.storage.partitions.len() as u32
    }

    /// VTK: `vtkPartitionedDataSet::GetPartition`.
    pub fn get_partition(&self, idx: u32) -> Option<&DataObject> {
        self.get_partition_as_data_object(idx)
            .filter(|partition| is_data_set_class(partition))
    }

    /// VTK: `vtkPartitionedDataSet::GetPartitionAsDataObject`.
    pub fn get_partition_as_data_object(&self, idx: u32) -> Option<&DataObject> {
        self.storage
            .partitions
            .get(idx as usize)
            .and_then(Option::as_ref)
    }

    /// VTK: `vtkPartitionedDataSet::SetPartition`.
    pub fn set_partition(&mut self, idx: u32, partition: Option<DataObject>) {
        if let Some(partition) = partition.as_ref() {
            if !Self::validate_partition(partition) {
                return;
            }
        }
        let idx = idx as usize;
        if idx >= self.storage.partitions.len() {
            self.set_number_of_partitions((idx + 1) as u32);
        }
        self.storage_mut().partitions[idx] = partition;
    }

    /// VTK: `vtkPartitionedDataSet::RemoveNullPartitions`.
    pub fn remove_null_partitions(&mut self) {
        let storage = self.storage_mut();
        storage
            .partitions
            .retain(|partition| partition.as_ref().is_some_and(is_data_set_class));
    }

    /// VTK: `vtkPartitionedDataSet::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkPartitionedDataSet\n  Number Of Partitions: {}",
            self.get_number_of_partitions()
        )
    }

    /// VTK: `vtkPartitionedDataSet::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.storage = Arc::new((*source.storage).clone());
    }

    /// VTK: `vtkPartitionedDataSet::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Arc::clone(&source.storage);
    }

    fn validate_partition(partition: &DataObject) -> bool {
        !is_composite_data_set_class(partition.get_class_name())
    }

    fn storage_mut(&mut self) -> &mut PartitionedDataSetStorage {
        Arc::make_mut(&mut self.storage)
    }
}

pub(crate) fn is_composite_data_set_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "vtkCompositeDataSet"
            | "vtkDataObjectTree"
            | "vtkMultiBlockDataSet"
            | "vtkMultiPieceDataSet"
            | "vtkPartitionedDataSet"
            | "vtkPartitionedDataSetCollection"
            | "vtkUniformGridAMR"
            | "vtkOverlappingAMR"
            | "vtkNonOverlappingAMR"
            | "vtkHierarchicalBoxDataSet"
    )
}

pub(crate) fn is_data_set_class(partition: &DataObject) -> bool {
    matches!(
        partition.get_class_name(),
        "vtkDataSet"
            | "vtkPointSet"
            | "vtkPolyData"
            | "vtkStructuredGrid"
            | "vtkUnstructuredGrid"
            | "vtkExplicitStructuredGrid"
            | "vtkImageData"
            | "vtkUniformGrid"
            | "vtkRectilinearGrid"
            | "vtkStructuredPoints"
            | "vtkHyperTreeGrid"
            | "vtkOverlappingAMR"
            | "vtkNonOverlappingAMR"
    )
}
