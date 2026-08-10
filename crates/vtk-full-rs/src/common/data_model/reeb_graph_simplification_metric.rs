use std::ffi::c_void;

use super::DataSet;
use crate::common::core::{Object, VtkIdType, VtkMTimeType};

/// VTK: `vtkDataArray*`.
pub type DataArrayHandle = *mut c_void;

/// VTK: `vtkAbstractArray*`.
pub type AbstractArrayHandle = *mut c_void;

/// VTK: `vtkReebGraphSimplificationMetric`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReebGraphSimplificationMetric {
    object: Object,
    lower_bound: f64,
    upper_bound: f64,
}

impl ReebGraphSimplificationMetric {
    /// VTK: `vtkReebGraphSimplificationMetric::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkReebGraphSimplificationMetric")
    }

    /// VTK: `vtkReebGraphSimplificationMetric::vtkReebGraphSimplificationMetric`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            lower_bound: 0.0,
            upper_bound: 1.0,
        }
    }

    /// VTK: `vtkReebGraphSimplificationMetric::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Upper Bound: {}\nLower Bound: {}\n",
            self.upper_bound, self.lower_bound
        )
    }

    /// VTK: `vtkReebGraphSimplificationMetric::SetLowerBound`.
    pub fn set_lower_bound(&mut self, lower_bound: f64) {
        if self.lower_bound != lower_bound {
            self.lower_bound = lower_bound;
            self.modified();
        }
    }

    /// VTK: `vtkReebGraphSimplificationMetric::GetLowerBound`.
    pub fn get_lower_bound(&self) -> f64 {
        self.lower_bound
    }

    /// VTK: `vtkReebGraphSimplificationMetric::SetUpperBound`.
    pub fn set_upper_bound(&mut self, upper_bound: f64) {
        if self.upper_bound != upper_bound {
            self.upper_bound = upper_bound;
            self.modified();
        }
    }

    /// VTK: `vtkReebGraphSimplificationMetric::GetUpperBound`.
    pub fn get_upper_bound(&self) -> f64 {
        self.upper_bound
    }

    /// VTK: `vtkReebGraphSimplificationMetric::ComputeMetric`.
    pub fn compute_metric(
        &mut self,
        _mesh: *mut DataSet,
        _field: DataArrayHandle,
        _start_critical_point: VtkIdType,
        _vertex_list: AbstractArrayHandle,
        _end_critical_point: VtkIdType,
    ) -> f64 {
        print!("too bad, wrong code\n");
        0.0
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

impl Default for ReebGraphSimplificationMetric {
    fn default() -> Self {
        Self::new()
    }
}
