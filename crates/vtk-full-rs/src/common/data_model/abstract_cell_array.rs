use std::ffi::c_void;

use crate::common::core::{IdList, Object, VtkIdType, VtkMTimeType};

/// VTK: `vtkAbstractCellArray*`.
pub type AbstractCellArrayHandle = *mut c_void;

/// VTK: `vtkAbstractCellArray`.
///
/// This stores the abstract VTK base-class state shared by concrete cell-array
/// implementations. The pure virtual API is represented by
/// `AbstractCellArrayApi`.
#[derive(Debug, Clone)]
pub struct AbstractCellArray {
    object: Object,
    temp_cell: IdList,
}

impl AbstractCellArray {
    /// VTK: `vtkAbstractCellArray::vtkAbstractCellArray`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            temp_cell: IdList::new(),
        }
    }

    /// VTK: `vtkAbstractCellArray::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkAbstractCellArray::GetCellAtId(vtkIdType, vtkIdType&, vtkIdType const*&)`.
    pub fn get_cell_at_id<A>(&mut self, array: &mut A, cell_id: VtkIdType) -> Vec<VtkIdType>
    where
        A: AbstractCellArrayApi + ?Sized,
    {
        array.get_cell_at_id_with_temp(cell_id, &mut self.temp_cell)
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl Default for AbstractCellArray {
    fn default() -> Self {
        Self::with_class_name("vtkAbstractCellArray")
    }
}

/// VTK pure virtual API for `vtkAbstractCellArray`.
pub trait AbstractCellArrayApi {
    /// VTK: `vtkAbstractCellArray::Initialize`.
    fn initialize(&mut self);

    /// VTK: `vtkAbstractCellArray::GetNumberOfCells`.
    fn get_number_of_cells(&self) -> VtkIdType;

    /// VTK: `vtkAbstractCellArray::GetNumberOfOffsets`.
    fn get_number_of_offsets(&self) -> VtkIdType;

    /// VTK: `vtkAbstractCellArray::GetOffset`.
    fn get_offset(&mut self, cell_id: VtkIdType) -> VtkIdType;

    /// VTK: `vtkAbstractCellArray::GetNumberOfConnectivityIds`.
    fn get_number_of_connectivity_ids(&self) -> VtkIdType;

    /// VTK: `vtkAbstractCellArray::IsStorageShareable`.
    fn is_storage_shareable(&self) -> bool;

    /// VTK: `vtkAbstractCellArray::IsHomogeneous`.
    fn is_homogeneous(&self) -> VtkIdType;

    /// VTK: `vtkAbstractCellArray::GetCellAtId(vtkIdType, vtkIdType&, vtkIdType const*&, vtkIdList*)`.
    fn get_cell_at_id_with_temp(
        &mut self,
        cell_id: VtkIdType,
        pt_ids: &mut IdList,
    ) -> Vec<VtkIdType>;

    /// VTK: `vtkAbstractCellArray::GetCellAtId(vtkIdType, vtkIdList*)`.
    fn get_cell_at_id_into_id_list(&mut self, cell_id: VtkIdType, pts: &mut IdList);

    /// VTK: `vtkAbstractCellArray::GetCellAtId(vtkIdType, vtkIdType&, vtkIdType*)`.
    fn get_cell_at_id_into_slice(&mut self, cell_id: VtkIdType, cell_points: &mut [VtkIdType]);

    /// VTK: `vtkAbstractCellArray::GetCellSize`.
    fn get_cell_size(&self, cell_id: VtkIdType) -> VtkIdType;

    /// VTK: `vtkAbstractCellArray::GetMaxCellSize`.
    fn get_max_cell_size(&mut self) -> i32;

    /// VTK: `vtkAbstractCellArray::DeepCopy`.
    fn deep_copy(&mut self, ca: AbstractCellArrayHandle);

    /// VTK: `vtkAbstractCellArray::ShallowCopy`.
    fn shallow_copy(&mut self, ca: AbstractCellArrayHandle);
}
