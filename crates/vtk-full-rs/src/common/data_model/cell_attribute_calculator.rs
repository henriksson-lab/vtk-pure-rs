use std::ffi::c_void;

use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkCellAttributeCalculator`.
///
/// This is the empty VTK base class for per-cell attribute calculators. The
/// `vtkCellMetadata` and `vtkCellAttribute` arguments are forward declarations
/// in VTK and remain opaque until those classes are translated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellAttributeCalculator {
    object: Object,
}

impl CellAttributeCalculator {
    /// VTK: `vtkCellAttributeCalculator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCellAttributeCalculator"),
        }
    }

    /// VTK: `vtkCellAttributeCalculator::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkCellAttributeCalculator::Prepare`.
    pub fn prepare(
        &mut self,
        cell: *mut c_void,
        field: *mut c_void,
    ) -> Option<CellAttributeCalculator> {
        self.prepare_for_grid(cell, field)
    }

    /// VTK: `vtkCellAttributeCalculator::PrepareForGrid`.
    pub fn prepare_for_grid(
        &mut self,
        _cell: *mut c_void,
        _field: *mut c_void,
    ) -> Option<CellAttributeCalculator> {
        None
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

impl Default for CellAttributeCalculator {
    fn default() -> Self {
        Self::new()
    }
}
