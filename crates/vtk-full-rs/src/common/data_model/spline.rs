use std::any::Any;

use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkSpline`.
///
/// This stores the abstract `vtkSpline` base state. VTK keeps interpolation
/// points in an internal `vtkPiecewiseFunction`; this translation mirrors the
/// point storage behavior needed by `vtkSpline` directly and leaves the full
/// public `vtkPiecewiseFunction` class for its own slice.
#[derive(Debug, Clone, PartialEq)]
pub struct Spline {
    object: Object,
    compute_time: VtkMTimeType,
    clamp_value: bool,
    intervals: Vec<f64>,
    coefficients: Vec<[f64; 4]>,
    left_constraint: i32,
    left_value: f64,
    right_constraint: i32,
    right_value: f64,
    points: Vec<[f64; 2]>,
    data_m_time: VtkMTimeType,
    closed: bool,
    parametric_range: [f64; 2],
}

impl Spline {
    /// VTK: `vtkSpline::vtkSpline`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            compute_time: 0,
            clamp_value: false,
            intervals: Vec::new(),
            coefficients: Vec::new(),
            left_constraint: 1,
            left_value: 0.0,
            right_constraint: 1,
            right_value: 0.0,
            points: Vec::new(),
            data_m_time: 0,
            closed: false,
            parametric_range: [-1.0, -1.0],
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkSpline::SetParametricRange`.
    pub fn set_parametric_range(&mut self, t_min: f64, mut t_max: f64) {
        if t_min != self.parametric_range[0] || t_max != self.parametric_range[1] {
            if t_min >= t_max {
                t_max = t_min + 1.0;
            }

            self.parametric_range = [t_min, t_max];
            self.modified();
        }
    }

    /// VTK: `vtkSpline::SetParametricRange`.
    pub fn set_parametric_range_from_slice(&mut self, t_range: [f64; 2]) {
        self.set_parametric_range(t_range[0], t_range[1]);
    }

    /// VTK: `vtkSpline::GetParametricRange`.
    pub fn get_parametric_range(&self) -> [f64; 2] {
        if self.parametric_range[0] != self.parametric_range[1] {
            self.parametric_range
        } else {
            self.points_range()
        }
    }

    /// VTK: `vtkSpline::SetClampValue`.
    pub fn set_clamp_value(&mut self, value: bool) {
        if self.clamp_value != value {
            self.clamp_value = value;
            self.modified();
        }
    }

    /// VTK: `vtkSpline::GetClampValue`.
    pub fn get_clamp_value(&self) -> bool {
        self.clamp_value
    }

    /// VTK: `vtkSpline::ClampValueOn`.
    pub fn clamp_value_on(&mut self) {
        self.set_clamp_value(true);
    }

    /// VTK: `vtkSpline::ClampValueOff`.
    pub fn clamp_value_off(&mut self) {
        self.set_clamp_value(false);
    }

    /// VTK: `vtkSpline::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> i32 {
        self.points.len() as i32
    }

    /// VTK: `vtkSpline::AddPoint`.
    pub fn add_point(&mut self, mut t: f64, x: f64) {
        if self.parametric_range[0] != self.parametric_range[1] {
            t = t.clamp(self.parametric_range[0], self.parametric_range[1]);
        }

        if let Some(index) = self.points.iter().position(|point| point[0] == t) {
            self.points.remove(index);
        }

        self.points.push([t, x]);
        self.sort_points_and_modified();
    }

    /// VTK: `vtkSpline::FillFromDataPointer`.
    pub fn fill_from_data_pointer(&mut self, nb: i32, data: &[f64]) {
        if nb <= 0 || data.len() < nb as usize * 2 {
            return;
        }

        self.points.clear();
        for pair in data.chunks_exact(2).take(nb as usize) {
            self.points.push([pair[0], pair[1]]);
        }
        self.sort_points_and_modified();
    }

    /// VTK: `vtkSpline::RemovePoint`.
    pub fn remove_point(&mut self, mut t: f64) {
        if self.parametric_range[0] != self.parametric_range[1] {
            t = t.clamp(self.parametric_range[0], self.parametric_range[1]);
        }

        if let Some(index) = self.points.iter().position(|point| point[0] == t) {
            self.points.remove(index);
            self.data_modified();
        }
    }

    /// VTK: `vtkSpline::RemoveAllPoints`.
    pub fn remove_all_points(&mut self) {
        self.points.clear();
        self.data_modified();
    }

    /// VTK: `vtkSpline::SetClosed`.
    pub fn set_closed(&mut self, value: bool) {
        if self.closed != value {
            self.closed = value;
            self.modified();
        }
    }

    /// VTK: `vtkSpline::GetClosed`.
    pub fn get_closed(&self) -> bool {
        self.closed
    }

    /// VTK: `vtkSpline::ClosedOn`.
    pub fn closed_on(&mut self) {
        self.set_closed(true);
    }

    /// VTK: `vtkSpline::ClosedOff`.
    pub fn closed_off(&mut self) {
        self.set_closed(false);
    }

    /// VTK: `vtkSpline::SetLeftConstraint`.
    pub fn set_left_constraint(&mut self, value: i32) {
        let value = value.clamp(0, 3);
        if self.left_constraint != value {
            self.left_constraint = value;
            self.modified();
        }
    }

    /// VTK: `vtkSpline::GetLeftConstraint`.
    pub fn get_left_constraint(&self) -> i32 {
        self.left_constraint
    }

    /// VTK: `vtkSpline::SetRightConstraint`.
    pub fn set_right_constraint(&mut self, value: i32) {
        let value = value.clamp(0, 3);
        if self.right_constraint != value {
            self.right_constraint = value;
            self.modified();
        }
    }

    /// VTK: `vtkSpline::GetRightConstraint`.
    pub fn get_right_constraint(&self) -> i32 {
        self.right_constraint
    }

    /// VTK: `vtkSpline::SetLeftValue`.
    pub fn set_left_value(&mut self, value: f64) {
        if self.left_value != value {
            self.left_value = value;
            self.modified();
        }
    }

    /// VTK: `vtkSpline::GetLeftValue`.
    pub fn get_left_value(&self) -> f64 {
        self.left_value
    }

    /// VTK: `vtkSpline::SetRightValue`.
    pub fn set_right_value(&mut self, value: f64) {
        if self.right_value != value {
            self.right_value = value;
            self.modified();
        }
    }

    /// VTK: `vtkSpline::GetRightValue`.
    pub fn get_right_value(&self) -> f64 {
        self.right_value
    }

    /// VTK: `vtkSpline::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time().max(self.data_m_time)
    }

    /// VTK: `vtkSpline::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.clamp_value = other.clamp_value;
        self.left_constraint = other.left_constraint;
        self.left_value = other.left_value;
        self.right_constraint = other.right_constraint;
        self.right_value = other.right_value;
        self.closed = other.closed;
        self.points = other.points.clone();
        self.data_modified();
    }

    /// VTK: `vtkSpline::ComputeLeftDerivative`.
    pub(crate) fn compute_left_derivative(&self) -> f64 {
        if self.points.len() < 2 {
            0.0
        } else {
            self.points[1][0] - self.points[0][0]
        }
    }

    /// VTK: `vtkSpline::ComputeRightDerivative`.
    pub(crate) fn compute_right_derivative(&self) -> f64 {
        if self.points.len() < 2 {
            0.0
        } else {
            let size = self.points.len();
            self.points[size - 1][0] - self.points[size - 2][0]
        }
    }

    /// VTK: `vtkSpline::FindIndex`.
    pub(crate) fn find_index(&self, size: usize, t: f64) -> usize {
        let mut index = 0;
        if size > 2 {
            let mut right_idx = size - 1;
            let mut center_idx = right_idx - size / 2;
            loop {
                if self.intervals[index] <= t && t <= self.intervals[center_idx] {
                    right_idx = center_idx;
                } else {
                    index = center_idx;
                }

                if index + 1 == right_idx {
                    break;
                }
                center_idx = index + (right_idx - index) / 2;
            }
        }
        index
    }

    pub(crate) fn points(&self) -> &[[f64; 2]] {
        &self.points
    }

    pub(crate) fn intervals(&self) -> &[f64] {
        &self.intervals
    }

    pub(crate) fn set_intervals(&mut self, intervals: Vec<f64>) {
        self.intervals = intervals;
    }

    pub(crate) fn coefficients(&self) -> &[[f64; 4]] {
        &self.coefficients
    }

    pub(crate) fn set_coefficients(&mut self, coefficients: Vec<[f64; 4]>) {
        self.coefficients = coefficients;
    }

    pub(crate) fn get_compute_time(&self) -> VtkMTimeType {
        self.compute_time
    }

    pub(crate) fn set_compute_time_to_m_time(&mut self) {
        self.compute_time = self.get_m_time();
    }

    pub(crate) fn get_parametric_range_storage(&self) -> [f64; 2] {
        self.parametric_range
    }

    fn points_range(&self) -> [f64; 2] {
        match (self.points.first(), self.points.last()) {
            (Some(first), Some(last)) => [first[0], last[0]],
            _ => [0.0, 0.0],
        }
    }

    fn sort_points_and_modified(&mut self) {
        self.points.sort_by(|left, right| {
            left[0]
                .partial_cmp(&right[0])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.data_modified();
    }

    fn data_modified(&mut self) {
        self.object.modified();
        self.data_m_time = self.object.get_m_time();
    }
}

/// Runtime polymorphic surface for VTK classes derived from `vtkSpline`.
pub trait SplineApi: std::fmt::Debug {
    /// Runtime type identity for VTK-style SafeDownCast behavior.
    fn as_any(&self) -> &dyn Any;

    /// Access the embedded `vtkSpline` base storage.
    fn spline(&self) -> &Spline;

    /// Mutable access to the embedded `vtkSpline` base storage.
    fn spline_mut(&mut self) -> &mut Spline;

    /// VTK: `vtkSpline::Compute`.
    fn compute(&mut self);

    /// VTK: `vtkSpline::Evaluate`.
    fn evaluate(&mut self, t: f64) -> f64;

    /// Clone support for VTK-style spline pointers stored behind trait objects.
    fn clone_box(&self) -> Box<dyn SplineApi>;

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str {
        self.spline().get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    fn modified(&mut self) {
        self.spline_mut().modified();
    }

    /// VTK: `vtkSpline::GetMTime`.
    fn get_m_time(&self) -> VtkMTimeType {
        self.spline().get_m_time()
    }

    /// VTK: `vtkSpline::SetParametricRange`.
    fn set_parametric_range(&mut self, t_min: f64, t_max: f64) {
        self.spline_mut().set_parametric_range(t_min, t_max);
    }

    /// VTK: `vtkSpline::GetParametricRange`.
    fn get_parametric_range(&self) -> [f64; 2] {
        self.spline().get_parametric_range()
    }

    /// VTK: `vtkSpline::SetClosed`.
    fn set_closed(&mut self, value: bool) {
        self.spline_mut().set_closed(value);
    }

    /// VTK: `vtkSpline::GetClosed`.
    fn get_closed(&self) -> bool {
        self.spline().get_closed()
    }

    /// VTK: `vtkSpline::SetLeftConstraint`.
    fn set_left_constraint(&mut self, value: i32) {
        self.spline_mut().set_left_constraint(value);
    }

    /// VTK: `vtkSpline::GetLeftConstraint`.
    fn get_left_constraint(&self) -> i32 {
        self.spline().get_left_constraint()
    }

    /// VTK: `vtkSpline::SetRightConstraint`.
    fn set_right_constraint(&mut self, value: i32) {
        self.spline_mut().set_right_constraint(value);
    }

    /// VTK: `vtkSpline::GetRightConstraint`.
    fn get_right_constraint(&self) -> i32 {
        self.spline().get_right_constraint()
    }

    /// VTK: `vtkSpline::SetLeftValue`.
    fn set_left_value(&mut self, value: f64) {
        self.spline_mut().set_left_value(value);
    }

    /// VTK: `vtkSpline::GetLeftValue`.
    fn get_left_value(&self) -> f64 {
        self.spline().get_left_value()
    }

    /// VTK: `vtkSpline::SetRightValue`.
    fn set_right_value(&mut self, value: f64) {
        self.spline_mut().set_right_value(value);
    }

    /// VTK: `vtkSpline::GetRightValue`.
    fn get_right_value(&self) -> f64 {
        self.spline().get_right_value()
    }

    /// VTK: `vtkSpline::RemoveAllPoints`.
    fn remove_all_points(&mut self) {
        self.spline_mut().remove_all_points();
    }

    /// VTK: `vtkSpline::AddPoint`.
    fn add_point(&mut self, t: f64, x: f64) {
        self.spline_mut().add_point(t, x);
    }
}

impl Clone for Box<dyn SplineApi> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
