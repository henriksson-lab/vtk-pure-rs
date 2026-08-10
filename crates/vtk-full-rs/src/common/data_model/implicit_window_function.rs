use std::sync::atomic::{AtomicBool, Ordering};

use crate::common::core::{Object, VtkMTimeType};
use crate::common::data_model::ImplicitFunctionHandle;

static IMPLICIT_FUNCTION_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);

/// VTK: `vtkImplicitWindowFunction`.
#[derive(Clone)]
pub struct ImplicitWindowFunction {
    object: Object,
    implicit_function: Option<ImplicitFunctionHandle>,
    window_range: [f64; 2],
    window_values: [f64; 2],
}

impl ImplicitWindowFunction {
    /// VTK: `vtkImplicitWindowFunction::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkImplicitWindowFunction"),
            implicit_function: None,
            window_range: [0.0, 1.0],
            window_values: [0.0, 1.0],
        }
    }

    /// VTK: `vtkImplicitWindowFunction::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let Some(implicit_function) = &self.implicit_function else {
            IMPLICIT_FUNCTION_WARNING_EMITTED.store(true, Ordering::Relaxed);
            return 0.0;
        };

        let mut value = implicit_function.evaluate_function(x);

        let diff1 = value - self.window_range[0];
        let diff2 = value - self.window_range[1];

        let mut scaled_range = (self.window_values[1] - self.window_values[0]) / 2.0;
        if scaled_range == 0.0 {
            scaled_range = 1.0;
        }

        if diff1 >= 0.0 && diff2 <= 0.0 {
            if diff1 <= -diff2 {
                value = diff1 / scaled_range + self.window_values[0];
            } else {
                value = -diff2 / scaled_range + self.window_values[0];
            }
        } else if diff1 < 0.0 {
            value = diff1 / scaled_range + self.window_values[0];
        } else {
            value = -diff2 / scaled_range + self.window_values[0];
        }

        value
    }

    /// VTK: `vtkImplicitWindowFunction::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        self.implicit_function
            .as_ref()
            .map(|implicit_function| implicit_function.evaluate_gradient(x))
            .unwrap_or([0.0; 3])
    }

    /// VTK: `vtkImplicitWindowFunction::SetImplicitFunction`.
    pub fn set_implicit_function(&mut self, implicit_function: Option<ImplicitFunctionHandle>) {
        let changed = match (&self.implicit_function, &implicit_function) {
            (Some(current), Some(next)) => !current.ptr_eq(next),
            (None, None) => false,
            _ => true,
        };

        if changed {
            self.implicit_function = implicit_function;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitWindowFunction::GetImplicitFunction`.
    pub fn get_implicit_function(&self) -> Option<ImplicitFunctionHandle> {
        self.implicit_function.clone()
    }

    /// VTK: `vtkImplicitWindowFunction::SetWindowRange`.
    pub fn set_window_range(&mut self, lower: f64, upper: f64) {
        self.set_window_range_array([lower, upper]);
    }

    /// VTK: `vtkImplicitWindowFunction::SetWindowRange`.
    pub fn set_window_range_array(&mut self, window_range: [f64; 2]) {
        if self.window_range != window_range {
            self.window_range = window_range;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitWindowFunction::GetWindowRange`.
    pub fn get_window_range(&self) -> [f64; 2] {
        self.window_range
    }

    /// VTK: `vtkImplicitWindowFunction::SetWindowValues`.
    pub fn set_window_values(&mut self, lower: f64, upper: f64) {
        self.set_window_values_array([lower, upper]);
    }

    /// VTK: `vtkImplicitWindowFunction::SetWindowValues`.
    pub fn set_window_values_array(&mut self, window_values: [f64; 2]) {
        if self.window_values != window_values {
            self.window_values = window_values;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitWindowFunction::GetWindowValues`.
    pub fn get_window_values(&self) -> [f64; 2] {
        self.window_values
    }

    /// VTK: `vtkImplicitWindowFunction::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        let mtime = self.object.get_m_time();
        self.implicit_function
            .as_ref()
            .map(|implicit_function| mtime.max(implicit_function.get_m_time()))
            .unwrap_or(mtime)
    }

    /// VTK: `vtkImplicitWindowFunction::PrintSelf`.
    pub fn print_self(&self) -> String {
        let implicit_function = self
            .implicit_function
            .as_ref()
            .map(|function| function.get_class_name())
            .unwrap_or("No implicit function defined.");
        format!(
            "Implicit Function: {}\nWindow Range: ({}, {})\nWindow Values: ({}, {})\n",
            implicit_function,
            self.window_range[0],
            self.window_range[1],
            self.window_values[0],
            self.window_values[1]
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkImplicitWindowFunction::ReportReferences`.
    #[allow(dead_code)]
    pub(crate) fn report_implicit_function_reference(&self) -> Option<ImplicitFunctionHandle> {
        self.implicit_function.clone()
    }
}

impl Default for ImplicitWindowFunction {
    fn default() -> Self {
        Self::new()
    }
}
