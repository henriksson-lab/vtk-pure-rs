use crate::common::core::{Object, VtkMTimeType};

use super::{GenericAdaptorCellHandle, GenericDataSetHandle, GenericSubdivisionErrorMetricApi};

/// Rust handle for `vtkGenericSubdivisionErrorMetric*` entries stored by
/// `vtkGenericCellTessellator`.
pub type GenericSubdivisionErrorMetricHandle = Box<dyn GenericSubdivisionErrorMetricApi>;

/// VTK: `vtkGenericCellTessellator`.
///
/// This translates the concrete base-class state and the non-pure methods from
/// `vtkGenericCellTessellator`. Concrete tessellation algorithms remain with
/// subclasses such as `vtkSimpleCellTessellator`.
pub struct GenericCellTessellator {
    object: Object,
    data_set: Option<GenericDataSetHandle>,
    error_metrics: Vec<GenericSubdivisionErrorMetricHandle>,
    max_errors: Vec<f64>,
    max_errors_capacity: usize,
    measurement: i32,
}

impl GenericCellTessellator {
    /// VTK: `vtkGenericCellTessellator::vtkGenericCellTessellator`.
    #[allow(dead_code)]
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            data_set: None,
            error_metrics: Vec::new(),
            max_errors: Vec::new(),
            max_errors_capacity: 0,
            measurement: 0,
        }
    }

    /// VTK: `vtkGenericCellTessellator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Measurement: {}\nErrorMetrics: {:?}\n",
            self.measurement,
            self.error_metrics
                .iter()
                .map(|metric| (&**metric) as *const dyn GenericSubdivisionErrorMetricApi)
                .collect::<Vec<_>>()
        )
    }

    /// VTK: `vtkGenericCellTessellator::SetErrorMetrics`.
    pub fn set_error_metrics(&mut self, error_metrics: Vec<GenericSubdivisionErrorMetricHandle>) {
        self.error_metrics = error_metrics;
        self.modified();
    }

    /// VTK: `vtkGenericCellTessellator::GetErrorMetrics`.
    pub fn get_error_metrics(&self) -> &[GenericSubdivisionErrorMetricHandle] {
        &self.error_metrics
    }

    /// VTK: `vtkGenericCellTessellator::GetMeasurement`.
    pub fn get_measurement(&self) -> i32 {
        self.measurement
    }

    /// VTK: `vtkGenericCellTessellator::SetMeasurement`.
    pub fn set_measurement(&mut self, measurement: i32) {
        if self.measurement != measurement {
            self.measurement = measurement;
            self.modified();
        }
    }

    /// VTK: `vtkGenericCellTessellator::InitErrorMetrics`.
    pub fn init_error_metrics(&mut self, data_set: GenericDataSetHandle) {
        self.initialize_data_set(data_set.clone());
        for metric in &mut self.error_metrics {
            metric.set_data_set(Some(data_set.clone()));
        }
        if self.measurement != 0 {
            self.reset_max_errors();
        }
    }

    pub(crate) fn initialize_data_set(&mut self, data_set: GenericDataSetHandle) {
        self.data_set = Some(data_set);
    }

    #[allow(dead_code)]
    pub(crate) fn get_data_set(&self) -> Option<GenericDataSetHandle> {
        self.data_set.clone()
    }

    /// VTK: `vtkGenericCellTessellator::GetMaxErrors`.
    pub fn get_max_errors(&self, errors: &mut [f64]) {
        assert!(errors.len() >= self.error_metrics.len(), "pre: valid_size");
        assert!(
            self.max_errors.len() >= self.error_metrics.len(),
            "pre: max_errors_initialized"
        );
        for (dst, src) in errors.iter_mut().zip(self.max_errors.iter()) {
            *dst = *src;
        }
    }

    /// VTK: `vtkGenericCellTessellator::RequiresEdgeSubdivision`.
    pub fn requires_edge_subdivision(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        alpha: f64,
    ) -> i32 {
        assert!(!left_point.is_empty(), "pre: leftPoint_exists");
        assert!(!mid_point.is_empty(), "pre: midPoint_exists");
        assert!(!right_point.is_empty(), "pre: rightPoint_exists");
        assert!(alpha > 0.0 && alpha < 1.0, "pre: clamped_alpha");

        for metric in &mut self.error_metrics {
            if metric.requires_edge_subdivision(left_point, mid_point, right_point, alpha) != 0 {
                return 1;
            }
        }
        0
    }

    /// VTK: `vtkGenericCellTessellator::UpdateMaxError`.
    pub fn update_max_error(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        alpha: f64,
    ) {
        assert!(!left_point.is_empty(), "pre: leftPoint_exists");
        assert!(!mid_point.is_empty(), "pre: midPoint_exists");
        assert!(!right_point.is_empty(), "pre: rightPoint_exists");
        assert!(alpha > 0.0 && alpha < 1.0, "pre: clamped_alpha");
        assert!(
            self.max_errors.len() >= self.error_metrics.len(),
            "pre: max_errors_initialized"
        );

        for (max_error, metric) in self.max_errors.iter_mut().zip(&mut self.error_metrics) {
            let error = metric.get_error(left_point, mid_point, right_point, alpha);
            assert!(error >= 0.0, "check: positive_error");
            *max_error = (*max_error).max(error);
        }
    }

    /// VTK: `vtkGenericCellTessellator::ResetMaxErrors`.
    pub fn reset_max_errors(&mut self) {
        let count = self.error_metrics.len();
        if count > self.max_errors_capacity {
            self.max_errors_capacity = count;
            self.max_errors.resize(self.max_errors_capacity, 0.0);
        }
        self.max_errors[..count].fill(0.0);
    }

    /// VTK: `vtkGenericCellTessellator::SetGenericCell`.
    pub fn set_generic_cell(&mut self, cell: GenericAdaptorCellHandle) {
        for metric in &mut self.error_metrics {
            metric.set_generic_cell(Some(cell.clone()));
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

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl std::fmt::Debug for GenericCellTessellator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericCellTessellator")
            .field("object", &self.object)
            .field("data_set", &self.data_set.as_ref().map(std::rc::Rc::as_ptr))
            .field("error_metrics_len", &self.error_metrics.len())
            .field("max_errors", &self.max_errors)
            .field("max_errors_capacity", &self.max_errors_capacity)
            .field("measurement", &self.measurement)
            .finish()
    }
}
