use super::{DataObject, PartitionedDataSet};

pub const VTK_MULTIPIECE_DATA_SET: i32 = 25;

/// VTK: `vtkMultiPieceDataSet`.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPieceDataSet {
    partitioned_data_set: PartitionedDataSet,
}

impl MultiPieceDataSet {
    /// VTK: `vtkMultiPieceDataSet::New`.
    pub fn new() -> Self {
        Self {
            partitioned_data_set: PartitionedDataSet::new(),
        }
    }

    /// VTK: `vtkMultiPieceDataSet::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        VTK_MULTIPIECE_DATA_SET
    }

    /// VTK: `vtkMultiPieceDataSet::SetNumberOfPieces`.
    pub fn set_number_of_pieces(&mut self, num_pieces: u32) {
        self.partitioned_data_set
            .set_number_of_partitions(num_pieces);
    }

    /// VTK: `vtkMultiPieceDataSet::GetNumberOfPieces`.
    pub fn get_number_of_pieces(&self) -> u32 {
        self.partitioned_data_set.get_number_of_partitions()
    }

    /// VTK: `vtkMultiPieceDataSet::GetPiece`.
    pub fn get_piece(&self, piece_no: u32) -> Option<&DataObject> {
        self.partitioned_data_set.get_partition(piece_no)
    }

    /// VTK: `vtkMultiPieceDataSet::GetPieceAsDataObject`.
    pub fn get_piece_as_data_object(&self, piece_no: u32) -> Option<&DataObject> {
        self.partitioned_data_set
            .get_partition_as_data_object(piece_no)
    }

    /// VTK: `vtkMultiPieceDataSet::SetPiece`.
    pub fn set_piece(&mut self, piece_no: u32, piece: Option<DataObject>) {
        self.partitioned_data_set.set_partition(piece_no, piece);
    }

    /// VTK: `vtkMultiPieceDataSet::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.partitioned_data_set.print_self()
    }

    /// VTK: `vtkPartitionedDataSet::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.partitioned_data_set
            .deep_copy(&source.partitioned_data_set);
    }

    /// VTK: `vtkPartitionedDataSet::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.partitioned_data_set
            .shallow_copy(&source.partitioned_data_set);
    }
}

impl Default for MultiPieceDataSet {
    fn default() -> Self {
        Self::new()
    }
}
