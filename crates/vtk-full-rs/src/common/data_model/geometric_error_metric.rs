use crate::common::core::VtkMTimeType;

use super::{
    GenericDataSetHandle, GenericSubdivisionErrorMetric, GenericSubdivisionErrorMetricApi,
};

/// VTK: `vtkGeometricErrorMetric`.
#[derive(Debug, Clone)]
pub struct GeometricErrorMetric {
    metric: GenericSubdivisionErrorMetric,
    absolute_geometric_tolerance: f64,
    smallest_size: f64,
    relative: i32,
}

impl GeometricErrorMetric {
    /// VTK: `vtkGeometricErrorMetric::New`.
    pub fn new() -> Self {
        Self {
            metric: GenericSubdivisionErrorMetric::with_class_name("vtkGeometricErrorMetric"),
            absolute_geometric_tolerance: 1.0,
            smallest_size: 1.0,
            relative: 0,
        }
    }

    /// VTK: `vtkGeometricErrorMetric::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = self.metric.print_self();
        result.push_str(&format!(
            "AbsoluteGeometricTolerance: {}\n",
            self.absolute_geometric_tolerance
        ));
        result.push_str(&format!("SmallestSize: {}\n", self.smallest_size));
        result.push_str(&format!("Relative: {}\n", self.relative));
        result
    }

    /// VTK: `vtkGeometricErrorMetric::GetAbsoluteGeometricTolerance`.
    pub fn get_absolute_geometric_tolerance(&self) -> f64 {
        self.absolute_geometric_tolerance
    }

    /// VTK: `vtkGeometricErrorMetric::SetAbsoluteGeometricTolerance`.
    pub fn set_absolute_geometric_tolerance(&mut self, value: f64) {
        assert!(value > 0.0, "pre: positive_value");
        self.relative = 0;
        if self.absolute_geometric_tolerance != value {
            self.absolute_geometric_tolerance = value;
            self.modified();
        }
    }

    /// VTK: `vtkGeometricErrorMetric::SetRelativeGeometricTolerance`.
    pub fn set_relative_geometric_tolerance(&mut self, value: f64, data_set: GenericDataSetHandle) {
        assert!(value > 0.0 && value < 1.0, "pre: valid_range_value");

        let data_set = data_set.borrow();
        let bounds = data_set.get_bounds();
        let mut smallest = bounds[1] - bounds[0];
        let mut length = bounds[3] - bounds[2];
        if length < smallest || smallest == 0.0 {
            smallest = length;
        }
        length = bounds[5] - bounds[4];
        if length < smallest || smallest == 0.0 {
            smallest = length;
        }
        length = data_set.get_length();
        if length < smallest || smallest == 0.0 {
            smallest = length;
        }
        if smallest == 0.0 {
            smallest = 1.0;
        }

        self.smallest_size = smallest;
        self.relative = 1;
        let tolerance = value * smallest;
        let tolerance = tolerance * tolerance;
        if self.absolute_geometric_tolerance != tolerance {
            self.absolute_geometric_tolerance = tolerance;
            self.modified();
        }
    }

    /// VTK: `vtkGeometricErrorMetric::GetRelative`.
    pub fn get_relative(&self) -> i32 {
        self.relative
    }

    /// VTK: `vtkGeometricErrorMetric::Distance2LinePoint`.
    fn distance2_line_point(x: &[f64; 3], y: &[f64; 3], z: &[f64; 3]) -> f64 {
        let mut u = [y[0] - x[0], y[1] - x[1], y[2] - x[2]];
        normalize(&mut u);

        let v = [z[0] - x[0], z[1] - x[1], z[2] - x[2]];
        let dot_uv = dot(u, v);
        let w = [
            v[0] - dot_uv * u[0],
            v[1] - dot_uv * u[1],
            v[2] - dot_uv * u[2],
        ];
        dot(w, w)
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetGenericCell`.
    pub fn set_generic_cell(&mut self, cell: Option<super::GenericAdaptorCellHandle>) {
        self.metric.set_generic_cell(cell);
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetGenericCell`.
    pub fn get_generic_cell(&self) -> Option<super::GenericAdaptorCellHandle> {
        self.metric.get_generic_cell()
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: Option<GenericDataSetHandle>) {
        self.metric.set_data_set(data_set);
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetDataSet`.
    pub fn get_data_set(&self) -> Option<GenericDataSetHandle> {
        self.metric.get_data_set()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.metric.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.metric.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.metric.get_m_time()
    }
}

impl Default for GeometricErrorMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericSubdivisionErrorMetricApi for GeometricErrorMetric {
    fn set_generic_cell(&mut self, cell: Option<super::GenericAdaptorCellHandle>) {
        self.set_generic_cell(cell);
    }

    fn get_generic_cell(&self) -> Option<super::GenericAdaptorCellHandle> {
        self.get_generic_cell()
    }

    fn set_data_set(&mut self, data_set: Option<GenericDataSetHandle>) {
        self.set_data_set(data_set);
    }

    fn get_data_set(&self) -> Option<GenericDataSetHandle> {
        self.get_data_set()
    }

    fn requires_edge_subdivision(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        _alpha: f64,
    ) -> i32 {
        assert!(left_point.len() >= 3, "pre: leftPoint_exists");
        assert!(mid_point.len() >= 3, "pre: midPoint_exists");
        assert!(right_point.len() >= 3, "pre: rightPoint_exists");

        if self.metric.is_geometry_linear() {
            return 0;
        }

        let left = [left_point[0], left_point[1], left_point[2]];
        let mid = [mid_point[0], mid_point[1], mid_point[2]];
        let right = [right_point[0], right_point[1], right_point[2]];
        (Self::distance2_line_point(&left, &right, &mid) > self.absolute_geometric_tolerance) as i32
    }

    fn get_error(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        _alpha: f64,
    ) -> f64 {
        assert!(left_point.len() >= 3, "pre: leftPoint_exists");
        assert!(mid_point.len() >= 3, "pre: midPoint_exists");
        assert!(right_point.len() >= 3, "pre: rightPoint_exists");

        if self.metric.is_geometry_linear() {
            return 0.0;
        }

        let left = [left_point[0], left_point[1], left_point[2]];
        let mid = [mid_point[0], mid_point[1], mid_point[2]];
        let right = [right_point[0], right_point[1], right_point[2]];
        let square_absolute_error = Self::distance2_line_point(&left, &right, &mid);
        if self.relative != 0 {
            square_absolute_error.sqrt() / self.smallest_size
        } else {
            square_absolute_error
        }
    }
}

fn normalize(vector: &mut [f64; 3]) -> f64 {
    let norm = dot(*vector, *vector).sqrt();
    if norm != 0.0 {
        vector[0] /= norm;
        vector[1] /= norm;
        vector[2] /= norm;
    }
    norm
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
