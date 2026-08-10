use super::{DataObject, PartitionedDataSet};
use std::sync::Arc;

/// Shared storage for collection slots.
#[derive(Debug, Clone, Default, PartialEq)]
struct PartitionedDataSetCollectionStorage {
    datasets: Vec<PartitionedDataSet>,
}

/// Collection of non-null partitioned datasets.
///
/// VTK origin:
/// `VTK/Common/DataModel/vtkPartitionedDataSetCollection.{h,cxx}`.
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedDataSetCollection {
    storage: Arc<PartitionedDataSetCollectionStorage>,
}

impl PartitionedDataSetCollection {
    /// VTK: `vtkPartitionedDataSetCollection::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(PartitionedDataSetCollectionStorage::default()),
        }
    }

    /// VTK: `vtkPartitionedDataSetCollection::SetNumberOfPartitionedDataSets`.
    pub fn set_number_of_partitioned_data_sets(&mut self, num_data_sets: u32) {
        self.storage_mut()
            .datasets
            .resize_with(num_data_sets as usize, PartitionedDataSet::new);
    }

    /// VTK: `vtkPartitionedDataSetCollection::GetNumberOfPartitionedDataSets`.
    pub fn get_number_of_partitioned_data_sets(&self) -> u32 {
        self.storage.datasets.len() as u32
    }

    /// VTK: `vtkPartitionedDataSetCollection::GetPartitionedDataSet`.
    pub fn get_partitioned_data_set(&self, idx: u32) -> Option<&PartitionedDataSet> {
        self.storage.datasets.get(idx as usize)
    }

    /// VTK: `vtkPartitionedDataSetCollection::SetPartitionedDataSet`.
    pub fn set_partitioned_data_set(&mut self, idx: u32, dataset: PartitionedDataSet) {
        let idx = idx as usize;
        if idx >= self.storage.datasets.len() {
            self.set_number_of_partitioned_data_sets((idx + 1) as u32);
        }
        self.storage_mut().datasets[idx] = dataset;
    }

    /// VTK: `vtkPartitionedDataSetCollection::RemovePartitionedDataSet`.
    pub fn remove_partitioned_data_set(&mut self, idx: u32) {
        let storage = self.storage_mut();
        let idx = idx as usize;
        if idx >= storage.datasets.len() {
            return;
        }
        storage.datasets.remove(idx);
    }

    /// VTK: `vtkPartitionedDataSetCollection::SetPartition`.
    pub fn set_partition(&mut self, idx: u32, partition: u32, object: Option<DataObject>) {
        let idx_usize = idx as usize;
        if idx_usize >= self.storage.datasets.len() {
            self.set_number_of_partitioned_data_sets(idx + 1);
        }
        self.storage_mut().datasets[idx_usize].set_partition(partition, object);
    }

    /// VTK: `vtkPartitionedDataSetCollection::GetPartition`.
    pub fn get_partition(&self, idx: u32, partition: u32) -> Option<&DataObject> {
        self.get_partitioned_data_set(idx)?.get_partition(partition)
    }

    /// VTK: `vtkPartitionedDataSetCollection::GetPartitionAsDataObject`.
    pub fn get_partition_as_data_object(&self, idx: u32, partition: u32) -> Option<&DataObject> {
        self.get_partitioned_data_set(idx)?
            .get_partition_as_data_object(partition)
    }

    /// VTK: `vtkPartitionedDataSetCollection::GetNumberOfPartitions`.
    pub fn get_number_of_partitions(&self, idx: u32) -> u32 {
        self.get_partitioned_data_set(idx)
            .map_or(0, PartitionedDataSet::get_number_of_partitions)
    }

    /// VTK: `vtkPartitionedDataSetCollection::SetNumberOfPartitions`.
    pub fn set_number_of_partitions(&mut self, idx: u32, num_partitions: u32) {
        let idx_usize = idx as usize;
        if idx_usize >= self.storage.datasets.len() {
            self.set_number_of_partitioned_data_sets(idx + 1);
        }
        self.storage_mut().datasets[idx_usize].set_number_of_partitions(num_partitions);
    }

    /// VTK: `vtkPartitionedDataSetCollection::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.storage = Arc::new((*source.storage).clone());
    }

    /// VTK: `vtkPartitionedDataSetCollection::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Arc::clone(&source.storage);
    }

    fn storage_mut(&mut self) -> &mut PartitionedDataSetCollectionStorage {
        Arc::make_mut(&mut self.storage)
    }
}
