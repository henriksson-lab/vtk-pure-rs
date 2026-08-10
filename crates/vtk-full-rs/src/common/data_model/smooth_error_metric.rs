use crate::common::core::VtkMTimeType;

use super::{GenericSubdivisionErrorMetric, GenericSubdivisionErrorMetricApi};

/// VTK: `vtkSmoothErrorMetric`.
#[derive(Debug, Clone)]
pub struct SmoothErrorMetric {
    metric: GenericSubdivisionErrorMetric,
    angle_tolerance: f64,
    cos_tolerance: f64,
}

impl SmoothErrorMetric {
    /// VTK: `vtkSmoothErrorMetric::New`.
    pub fn new() -> Self {
        let angle_tolerance: f64 = 90.1;
        Self {
            metric: GenericSubdivisionErrorMetric::with_class_name("vtkSmoothErrorMetric"),
            angle_tolerance,
            cos_tolerance: angle_tolerance.to_radians().cos(),
        }
    }

    /// VTK: `vtkSmoothErrorMetric::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = self.metric.print_self();
        result.push_str(&format!("AngleTolerance: {}\n", self.angle_tolerance));
        result.push_str(&format!("CosTolerance: {}\n", self.cos_tolerance));
        result
    }

    /// VTK: `vtkSmoothErrorMetric::GetAngleTolerance`.
    pub fn get_angle_tolerance(&self) -> f64 {
        self.angle_tolerance
    }

    /// VTK: `vtkSmoothErrorMetric::SetAngleTolerance`.
    pub fn set_angle_tolerance(&mut self, value: f64) {
        if self.angle_tolerance == value {
            return;
        }

        self.angle_tolerance = if value <= 90.0 {
            90.1
        } else if value >= 180.0 {
            179.9
        } else {
            value
        };
        self.cos_tolerance = self.angle_tolerance.to_radians().cos();
        self.modified();
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
    pub fn set_data_set(&mut self, data_set: Option<super::GenericDataSetHandle>) {
        self.metric.set_data_set(data_set);
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetDataSet`.
    pub fn get_data_set(&self) -> Option<super::GenericDataSetHandle> {
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

    fn is_geometry_linear(&self) -> bool {
        self.metric
            .get_generic_cell()
            .expect("pre: generic_cell_exists")
            .borrow()
            .is_geometry_linear()
    }

    fn edge_cosine(left_point: &[f64], mid_point: &[f64], right_point: &[f64]) -> f64 {
        assert!(left_point.len() >= 3, "pre: valid_size");
        assert!(mid_point.len() >= 3, "pre: valid_size");
        assert!(right_point.len() >= 3, "pre: valid_size");
        let a = [
            left_point[0] - mid_point[0],
            left_point[1] - mid_point[1],
            left_point[2] - mid_point[2],
        ];
        let b = [
            right_point[0] - mid_point[0],
            right_point[1] - mid_point[1],
            right_point[2] - mid_point[2],
        ];
        let dota = dot(a, a);
        let dotb = dot(b, b);
        if dota == 0.0 || dotb == 0.0 {
            -1.0
        } else {
            dot(a, b) / (dota * dotb).sqrt()
        }
    }
}

impl Default for SmoothErrorMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericSubdivisionErrorMetricApi for SmoothErrorMetric {
    fn set_generic_cell(&mut self, cell: Option<super::GenericAdaptorCellHandle>) {
        self.set_generic_cell(cell);
    }

    fn get_generic_cell(&self) -> Option<super::GenericAdaptorCellHandle> {
        self.get_generic_cell()
    }

    fn set_data_set(&mut self, data_set: Option<super::GenericDataSetHandle>) {
        self.set_data_set(data_set);
    }

    fn get_data_set(&self) -> Option<super::GenericDataSetHandle> {
        self.get_data_set()
    }

    fn requires_edge_subdivision(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        _alpha: f64,
    ) -> i32 {
        assert!(!left_point.is_empty(), "pre: leftPoint_exists");
        assert!(!mid_point.is_empty(), "pre: midPoint_exists");
        assert!(!right_point.is_empty(), "pre: rightPoint_exists");
        if self.is_geometry_linear() {
            return 0;
        }

        (Self::edge_cosine(left_point, mid_point, right_point) > self.cos_tolerance) as i32
    }

    fn get_error(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        _alpha: f64,
    ) -> f64 {
        assert!(!left_point.is_empty(), "pre: leftPoint_exists");
        assert!(!mid_point.is_empty(), "pre: midPoint_exists");
        assert!(!right_point.is_empty(), "pre: rightPoint_exists");
        if self.is_geometry_linear() {
            return 0.0;
        }

        let cosa = Self::edge_cosine(left_point, mid_point, right_point).clamp(-1.0, 1.0);
        180.0 - cosa.acos().to_radians()
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
