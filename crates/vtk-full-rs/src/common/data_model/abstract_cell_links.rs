use std::{ffi::c_void, ptr};

use crate::common::core::{Object, TimeStamp, VtkIdType, VtkMTimeType};

use super::{CellArray, DataSet};

/// VTK: `vtkAbstractCellLinks*`.
pub type AbstractCellLinksHandle = *mut c_void;

/// VTK: `vtkAbstractCellLinks::CellLinksTypes`.
pub struct CellLinksTypes;

impl CellLinksTypes {
    /// VTK: `vtkAbstractCellLinks::LINKS_NOT_DEFINED`.
    pub const LINKS_NOT_DEFINED: i32 = 0;
    /// VTK: `vtkAbstractCellLinks::CELL_LINKS`.
    pub const CELL_LINKS: i32 = 1;
    /// VTK: `vtkAbstractCellLinks::STATIC_CELL_LINKS_USHORT`.
    pub const STATIC_CELL_LINKS_USHORT: i32 = 2;
    /// VTK: `vtkAbstractCellLinks::STATIC_CELL_LINKS_UINT`.
    pub const STATIC_CELL_LINKS_UINT: i32 = 3;
    /// VTK: `vtkAbstractCellLinks::STATIC_CELL_LINKS_IDTYPE`.
    pub const STATIC_CELL_LINKS_IDTYPE: i32 = 4;
    /// VTK: `vtkAbstractCellLinks::STATIC_CELL_LINKS_SPECIALIZED`.
    pub const STATIC_CELL_LINKS_SPECIALIZED: i32 = 5;
}

/// VTK: `vtkAbstractCellLinks`.
///
/// This stores the abstract VTK base-class state shared by concrete cell-link
/// implementations. The pure virtual API is represented by
/// `AbstractCellLinksApi`.
#[derive(Debug, Clone)]
pub struct AbstractCellLinks {
    object: Object,
    data_set: *mut DataSet,
    cell_links_type: i32,
    build_time: TimeStamp,
}

impl AbstractCellLinks {
    /// VTK: `vtkAbstractCellLinks::vtkAbstractCellLinks`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            data_set: ptr::null_mut(),
            cell_links_type: CellLinksTypes::LINKS_NOT_DEFINED,
            build_time: TimeStamp::new(),
        }
    }

    pub(crate) fn set_type(&mut self, cell_links_type: i32) {
        self.cell_links_type = cell_links_type;
    }

    pub(crate) fn build_time_modified(&mut self) {
        self.build_time.modified();
    }

    /// VTK: `vtkAbstractCellLinks::~vtkAbstractCellLinks`.
    pub fn clear_data_set_on_drop_path(&mut self) {
        self.set_data_set(ptr::null_mut());
    }

    /// VTK: `vtkAbstractCellLinks::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut DataSet) {
        if self.data_set != data_set {
            self.data_set = data_set;
            self.modified();
        }
    }

    /// VTK: `vtkAbstractCellLinks::GetDataSet`.
    pub fn get_data_set(&self) -> *mut DataSet {
        self.data_set
    }

    /// VTK: `vtkAbstractCellLinks::ComputeType(vtkIdType, vtkIdType, vtkCellArray*)`.
    pub fn compute_type(max_pt_id: VtkIdType, max_cell_id: VtkIdType, ca: &CellArray) -> i32 {
        Self::compute_type_from_connectivity_size(
            max_pt_id,
            max_cell_id,
            ca.get_number_of_connectivity_ids(),
        )
    }

    /// VTK: `vtkAbstractCellLinks::ComputeType(vtkIdType, vtkIdType, vtkIdType)`.
    pub fn compute_type_from_connectivity_size(
        max_pt_id: VtkIdType,
        max_cell_id: VtkIdType,
        connectivity_size: VtkIdType,
    ) -> i32 {
        let max_id = max_pt_id.max(max_cell_id).max(connectivity_size);
        if max_id < u16::MAX as VtkIdType {
            CellLinksTypes::STATIC_CELL_LINKS_USHORT
        } else if max_id < u32::MAX as VtkIdType {
            CellLinksTypes::STATIC_CELL_LINKS_UINT
        } else {
            CellLinksTypes::STATIC_CELL_LINKS_IDTYPE
        }
    }

    /// VTK: `vtkAbstractCellLinks::PrintSelf`.
    pub fn print_self(&self) -> String {
        let data_set = if self.data_set.is_null() {
            "(none)".to_string()
        } else {
            format!("{:?}", self.data_set)
        };
        format!("DataSet: {data_set}\nType: {}\n", self.cell_links_type)
    }

    /// VTK: `vtkAbstractCellLinks::GetType`.
    pub fn get_type(&self) -> i32 {
        self.cell_links_type
    }

    /// VTK: `vtkAbstractCellLinks::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.build_time.get_m_time()
    }

    /// VTK: `vtkAbstractCellLinks::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        true
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

impl Default for AbstractCellLinks {
    fn default() -> Self {
        Self::with_class_name("vtkAbstractCellLinks")
    }
}

impl Drop for AbstractCellLinks {
    fn drop(&mut self) {
        self.clear_data_set_on_drop_path();
    }
}

/// VTK pure virtual API for `vtkAbstractCellLinks`.
pub trait AbstractCellLinksApi {
    /// VTK: `vtkAbstractCellLinks::BuildLinks`.
    fn build_links(&mut self);

    /// VTK: `vtkAbstractCellLinks::Initialize`.
    fn initialize(&mut self);

    /// VTK: `vtkAbstractCellLinks::Squeeze`.
    fn squeeze(&mut self);

    /// VTK: `vtkAbstractCellLinks::Reset`.
    fn reset(&mut self);

    /// VTK: `vtkAbstractCellLinks::GetActualMemorySize`.
    fn get_actual_memory_size(&mut self) -> u64;

    /// VTK: `vtkAbstractCellLinks::DeepCopy`.
    fn deep_copy(&mut self, src: AbstractCellLinksHandle);

    /// VTK: `vtkAbstractCellLinks::ShallowCopy`.
    fn shallow_copy(&mut self, src: AbstractCellLinksHandle);

    /// VTK: `vtkAbstractCellLinks::SelectCells`.
    fn select_cells(&mut self, min_max_degree: [VtkIdType; 2], cell_selection: &mut [u8]);
}
