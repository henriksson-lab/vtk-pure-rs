use std::ffi::c_void;

use super::PointSet;
use crate::common::core::{Object, TimeStamp, VtkIdType, VtkMTimeType};

/// VTK: `vtkCell*`.
pub type CellHandle = *mut c_void;

/// VTK: `vtkGenericCell*`.
pub type GenericCellHandle = *mut c_void;

/// VTK virtual cell API used by `vtkFindCellStrategy::FindCell`.
pub trait CellApi {
    /// VTK: `vtkCell::EvaluatePosition`.
    #[allow(clippy::too_many_arguments)]
    fn evaluate_position(
        &mut self,
        x: [f64; 3],
        closest_point: &mut [f64; 3],
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        dist2: &mut f64,
        weights: &mut [f64],
    ) -> i32;
}

/// VTK: `vtkFindCellStrategy`.
#[derive(Debug, Clone, PartialEq)]
pub struct FindCellStrategy {
    object: Object,
    owns_locator: bool,
    is_a_copy: bool,
    point_set: *mut PointSet,
    bounds: [f64; 6],
    initialize_time: TimeStamp,
}

impl FindCellStrategy {
    /// VTK: `vtkFindCellStrategy::vtkFindCellStrategy`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            owns_locator: false,
            is_a_copy: false,
            point_set: std::ptr::null_mut(),
            bounds: [0.0; 6],
            initialize_time: TimeStamp::new(),
        }
    }

    /// VTK: `vtkFindCellStrategy::Initialize`.
    pub fn initialize(&mut self, ps: Option<&mut PointSet>) -> i32 {
        let Some(ps) = ps else {
            return 0;
        };
        if ps.get_points().is_none() || ps.get_number_of_points() < 1 {
            return 0;
        }

        self.point_set = ps as *mut PointSet;
        self.bounds = ps.get_bounds();
        1
    }

    /// VTK: `vtkFindCellStrategy::CopyParameters`.
    pub fn copy_parameters(&mut self, from: &Self) {
        self.point_set = from.point_set;
        self.bounds = from.bounds;
        self.is_a_copy = true;
    }

    /// VTK: `vtkFindCellStrategy::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkPointSet: {}\n",
            if self.point_set.is_null() {
                "(none)"
            } else {
                "set"
            }
        )
    }

    /// VTK protected field: `OwnsLocator`.
    pub(crate) fn get_owns_locator(&self) -> bool {
        self.owns_locator
    }

    /// VTK protected field: `OwnsLocator`.
    pub(crate) fn set_owns_locator(&mut self, owns_locator: bool) {
        self.owns_locator = owns_locator;
    }

    /// VTK protected field: `IsACopy`.
    pub(crate) fn get_is_a_copy(&self) -> bool {
        self.is_a_copy
    }

    /// VTK protected field: `PointSet`.
    pub(crate) fn get_point_set(&self) -> *mut PointSet {
        self.point_set
    }

    /// VTK protected field: `Bounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.bounds
    }

    /// VTK protected field: `InitializeTime`.
    pub fn get_initialize_time(&self) -> VtkMTimeType {
        self.initialize_time.get_m_time()
    }

    /// VTK: `vtkTimeStamp::Modified` on `InitializeTime`.
    pub(crate) fn initialize_time_modified(&mut self) {
        self.initialize_time.modified();
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

impl Default for FindCellStrategy {
    fn default() -> Self {
        Self::with_class_name("vtkFindCellStrategy")
    }
}

/// VTK pure virtual search API for `vtkFindCellStrategy`.
pub trait FindCellStrategyApi {
    fn find_cell_strategy(&self) -> &FindCellStrategy;
    fn find_cell_strategy_mut(&mut self) -> &mut FindCellStrategy;

    /// VTK: `vtkFindCellStrategy::FindCell`.
    #[allow(clippy::too_many_arguments)]
    fn find_cell(
        &mut self,
        x: [f64; 3],
        cell: Option<&mut dyn CellApi>,
        gencell: GenericCellHandle,
        cell_id: VtkIdType,
        tol2: f64,
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: &mut [f64],
    ) -> VtkIdType;

    /// VTK: `vtkFindCellStrategy::FindClosestPointWithinRadius`.
    #[allow(clippy::too_many_arguments)]
    fn find_closest_point_within_radius(
        &mut self,
        x: [f64; 3],
        radius: f64,
        closest_point: &mut [f64; 3],
        cell: GenericCellHandle,
        cell_id: &mut VtkIdType,
        sub_id: &mut i32,
        dist2: &mut f64,
        inside: &mut i32,
    ) -> VtkIdType;

    /// VTK: `vtkFindCellStrategy::InsideCellBounds`.
    fn inside_cell_bounds(&self, x: [f64; 3], cell_id: VtkIdType) -> bool;

    /// VTK: `vtkFindCellStrategy::Initialize`.
    fn initialize(&mut self, ps: Option<&mut PointSet>) -> i32 {
        self.find_cell_strategy_mut().initialize(ps)
    }

    /// VTK: `vtkFindCellStrategy::CopyParameters`.
    fn copy_parameters(&mut self, from: &FindCellStrategy) {
        self.find_cell_strategy_mut().copy_parameters(from);
    }
}
