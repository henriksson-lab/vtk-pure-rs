use std::ffi::c_void;

use super::find_cell_strategy::GenericCellHandle;
use crate::common::core::{IdList, Object, Points, VtkIdType, VtkMTimeType};

/// VTK: `vtkHyperTreeGrid*`.
pub type HyperTreeGridHandle = *mut c_void;

/// VTK: `vtkHyperTreeGridLocator`.
#[derive(Debug, Clone, PartialEq)]
pub struct HyperTreeGridLocator {
    object: Object,
    htg: HyperTreeGridHandle,
    tolerance: f64,
}

impl HyperTreeGridLocator {
    /// VTK: `vtkHyperTreeGridLocator::vtkHyperTreeGridLocator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            htg: std::ptr::null_mut(),
            tolerance: 0.0,
        }
    }

    /// VTK: `vtkHyperTreeGridLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "HyperTreeGrid: {}\n",
            if self.htg.is_null() { "none" } else { "set" }
        )
    }

    /// VTK: `vtkHyperTreeGridLocator::GetHTG`.
    pub fn get_htg(&self) -> HyperTreeGridHandle {
        self.htg
    }

    /// VTK: `vtkHyperTreeGridLocator::SetHTG`.
    pub fn set_htg(&mut self, htg: HyperTreeGridHandle) {
        if self.htg != htg {
            self.htg = htg;
            self.modified();
        }
    }

    /// VTK: `vtkHyperTreeGridLocator::Initialize`.
    pub fn initialize(&mut self) {}

    /// VTK: `vtkHyperTreeGridLocator::Update`.
    pub fn update(&mut self) {
        if self.htg.is_null() {
            return;
        }
    }

    /// VTK: `vtkHyperTreeGridLocator::SetTolerance`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        if self.tolerance != tolerance {
            self.tolerance = tolerance;
            self.modified();
        }
    }

    /// VTK: `vtkHyperTreeGridLocator::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.tolerance
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

impl Default for HyperTreeGridLocator {
    fn default() -> Self {
        Self::with_class_name("vtkHyperTreeGridLocator")
    }
}

/// VTK pure virtual search API for `vtkHyperTreeGridLocator`.
pub trait HyperTreeGridLocatorApi {
    fn hyper_tree_grid_locator(&self) -> &HyperTreeGridLocator;
    fn hyper_tree_grid_locator_mut(&mut self) -> &mut HyperTreeGridLocator;

    /// VTK: `vtkHyperTreeGridLocator::Search`.
    fn search(&mut self, point: [f64; 3]) -> VtkIdType;

    /// VTK: `vtkHyperTreeGridLocator::FindCell`.
    fn find_cell(
        &mut self,
        point: [f64; 3],
        tol: f64,
        cell: GenericCellHandle,
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: &mut [f64],
    ) -> VtkIdType;

    /// VTK: `vtkHyperTreeGridLocator::IntersectWithLine` first-hit overload.
    #[allow(clippy::too_many_arguments)]
    fn intersect_with_line(
        &mut self,
        p0: [f64; 3],
        p1: [f64; 3],
        tol: f64,
        t: &mut f64,
        x: &mut [f64; 3],
        pcoords: &mut [f64; 3],
        sub_id: &mut i32,
        cell_id: &mut VtkIdType,
        cell: GenericCellHandle,
    ) -> i32;

    /// VTK: `vtkHyperTreeGridLocator::IntersectWithLine` all-hits overload.
    fn intersect_with_line_all(
        &mut self,
        p0: [f64; 3],
        p1: [f64; 3],
        tol: f64,
        points: &mut Points,
        cell_ids: &mut IdList,
        cell: GenericCellHandle,
    ) -> i32;

    /// VTK: `vtkHyperTreeGridLocator::Initialize`.
    fn initialize(&mut self) {
        self.hyper_tree_grid_locator_mut().initialize();
    }

    /// VTK: `vtkHyperTreeGridLocator::Update`.
    fn update(&mut self) {
        self.hyper_tree_grid_locator_mut().update();
    }
}
