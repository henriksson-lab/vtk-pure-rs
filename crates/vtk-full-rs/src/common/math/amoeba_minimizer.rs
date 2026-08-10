use std::ffi::c_void;
use std::ptr;

use crate::common::core::object::Object;

const N_STEPS_NO_VALUE_IMPROVEMENT: i32 = 2;
const N_STEPS_NO_PARAM_IMPROVEMENT: i32 = 18;
const VTK_AMOEBA_SMALLEST: f64 = 1.0e-20;

pub type AmoebaCallback = fn(*mut c_void);

/// VTK: `vtkAmoebaMinimizer`.
pub struct AmoebaMinimizer {
    object: Object,
    function: Option<AmoebaCallback>,
    function_arg_delete: Option<AmoebaCallback>,
    function_arg: *mut c_void,
    parameter_names: Vec<Option<String>>,
    parameter_values: Vec<f64>,
    parameter_scales: Vec<f64>,
    function_value: f64,
    contraction_ratio: f64,
    expansion_ratio: f64,
    tolerance: f64,
    parameter_tolerance: f64,
    max_iterations: i32,
    iterations: i32,
    function_evaluations: i32,
    amoeba_vertices: Vec<Vec<f64>>,
    amoeba_values: Vec<f64>,
    amoeba_sum: Vec<f64>,
    amoeba_size: f64,
    amoeba_high_value: f64,
    amoeba_n_steps_no_improvement: i32,
}

impl AmoebaMinimizer {
    /// VTK: `vtkAmoebaMinimizer::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkAmoebaMinimizer"),
            function: None,
            function_arg_delete: None,
            function_arg: ptr::null_mut(),
            parameter_names: Vec::new(),
            parameter_values: Vec::new(),
            parameter_scales: Vec::new(),
            function_value: 0.0,
            contraction_ratio: 0.5,
            expansion_ratio: 2.0,
            tolerance: 1e-4,
            parameter_tolerance: 1e-4,
            max_iterations: 1000,
            iterations: 0,
            function_evaluations: 0,
            amoeba_vertices: Vec::new(),
            amoeba_values: Vec::new(),
            amoeba_sum: Vec::new(),
            amoeba_size: 0.0,
            amoeba_high_value: 0.0,
            amoeba_n_steps_no_improvement: 0,
        }
    }

    /// VTK: `vtkAmoebaMinimizer::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = format!(
            "{}\nNumberOfParameters: {}\n",
            self.object.get_class_name(),
            self.get_number_of_parameters()
        );
        if !self.parameter_values.is_empty() {
            out.push_str("ParameterValues: \n");
            for i in 0..self.parameter_values.len() {
                let label = self
                    .parameter_names
                    .get(i)
                    .and_then(|name| name.as_deref())
                    .map_or_else(|| i.to_string(), ToOwned::to_owned);
                out.push_str(&format!("  {label}: {}\n", self.parameter_values[i]));
            }

            out.push_str("ParameterScales: \n");
            for i in 0..self.parameter_scales.len() {
                let label = self
                    .parameter_names
                    .get(i)
                    .and_then(|name| name.as_deref())
                    .map_or_else(|| i.to_string(), ToOwned::to_owned);
                out.push_str(&format!("  {label}: {}\n", self.parameter_scales[i]));
            }
        }

        out.push_str(&format!(
            "FunctionValue: {}\nFunctionEvaluations: {}\nIterations: {}\nMaxIterations: {}\nTolerance: {}\nParameterTolerance: {}\nContractionRatio: {}\nExpansionRatio: {}",
            self.get_function_value(),
            self.get_function_evaluations(),
            self.get_iterations(),
            self.get_max_iterations(),
            self.get_tolerance(),
            self.get_parameter_tolerance(),
            self.get_contraction_ratio(),
            self.get_expansion_ratio()
        ));
        out
    }

    /// VTK: `vtkAmoebaMinimizer::SetFunction`.
    pub fn set_function(&mut self, function: Option<AmoebaCallback>, arg: *mut c_void) {
        if callback_id(function) != callback_id(self.function) || arg != self.function_arg {
            self.delete_function_arg();
            self.function = function;
            self.function_arg = arg;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::SetFunctionArgDelete`.
    pub fn set_function_arg_delete(&mut self, function: Option<AmoebaCallback>) {
        if callback_id(function) != callback_id(self.function_arg_delete) {
            self.function_arg_delete = function;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::SetParameterValue`.
    pub fn set_parameter_value(&mut self, name: &str, value: f64) {
        let i = self
            .parameter_names
            .iter()
            .position(|parameter_name| parameter_name.as_deref() == Some(name))
            .unwrap_or(self.parameter_names.len());

        self.set_parameter_value_by_index(i as i32, value);

        if self.parameter_names[i].is_none() {
            self.parameter_names[i] = Some(name.to_owned());
        }
    }

    /// VTK: `vtkAmoebaMinimizer::SetParameterValue`.
    pub fn set_parameter_value_by_index(&mut self, i: i32, value: f64) {
        if i < 0 {
            return;
        }

        let i = i as usize;
        if i < self.parameter_values.len() {
            if self.parameter_values[i] != value {
                self.parameter_values[i] = value;
                self.iterations = 0;
                self.function_evaluations = 0;
                self.object.modified();
            }
            return;
        }

        let mut new_parameter_names = self.parameter_names.clone();
        let mut new_parameter_values = self.parameter_values.clone();
        let mut new_parameter_scales = self.parameter_scales.clone();
        new_parameter_names.push(None);
        new_parameter_values.push(value);
        new_parameter_scales.push(1.0);

        self.initialize();
        self.parameter_names = new_parameter_names;
        self.parameter_values = new_parameter_values;
        self.parameter_scales = new_parameter_scales;
        self.iterations = 0;
        self.function_evaluations = 0;
    }

    /// VTK: `vtkAmoebaMinimizer::SetParameterScale`.
    pub fn set_parameter_scale(&mut self, name: &str, scale: f64) {
        if let Some(i) = self
            .parameter_names
            .iter()
            .position(|parameter_name| parameter_name.as_deref() == Some(name))
        {
            self.set_parameter_scale_by_index(i as i32, scale);
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetParameterScale`.
    pub fn get_parameter_scale(&self, name: &str) -> f64 {
        self.parameter_names
            .iter()
            .position(|parameter_name| parameter_name.as_deref() == Some(name))
            .map_or(1.0, |i| self.parameter_scales[i])
    }

    /// VTK: `vtkAmoebaMinimizer::SetParameterScale`.
    pub fn set_parameter_scale_by_index(&mut self, i: i32, scale: f64) {
        if i < 0 || i as usize >= self.parameter_scales.len() {
            return;
        }

        let i = i as usize;
        if self.parameter_scales[i] != scale {
            self.parameter_scales[i] = scale;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetParameterScale`.
    pub fn get_parameter_scale_by_index(&self, i: i32) -> f64 {
        if i < 0 {
            return 1.0;
        }
        self.parameter_scales
            .get(i as usize)
            .copied()
            .unwrap_or(1.0)
    }

    /// VTK: `vtkAmoebaMinimizer::GetParameterValue`.
    pub fn get_parameter_value(&self, name: &str) -> f64 {
        self.parameter_names
            .iter()
            .position(|parameter_name| parameter_name.as_deref() == Some(name))
            .map_or(0.0, |i| self.parameter_values[i])
    }

    /// VTK: `vtkAmoebaMinimizer::GetParameterValue`.
    pub fn get_parameter_value_by_index(&self, i: i32) -> f64 {
        if i < 0 {
            return 0.0;
        }
        self.parameter_values
            .get(i as usize)
            .copied()
            .unwrap_or(0.0)
    }

    /// VTK: `vtkAmoebaMinimizer::GetParameterName`.
    pub fn get_parameter_name(&self, i: i32) -> Option<&str> {
        if i < 0 {
            return None;
        }
        self.parameter_names
            .get(i as usize)
            .and_then(|name| name.as_deref())
    }

    /// VTK: `vtkAmoebaMinimizer::GetNumberOfParameters`.
    pub fn get_number_of_parameters(&self) -> i32 {
        self.parameter_values.len() as i32
    }

    /// VTK: `vtkAmoebaMinimizer::Initialize`.
    pub fn initialize(&mut self) {
        self.parameter_names.clear();
        self.parameter_values.clear();
        self.parameter_scales.clear();
        self.iterations = 0;
        self.function_evaluations = 0;
        self.amoeba_size = 0.0;
        self.object.modified();
    }

    /// VTK: `vtkAmoebaMinimizer::Minimize`.
    pub fn minimize(&mut self) {
        if self.iterations == 0 {
            if self.function.is_none() {
                return;
            }
            self.initialize_amoeba();
        }

        while self.iterations < self.max_iterations {
            let improved = self.perform_amoeba();
            if !improved && self.check_parameter_tolerance() {
                break;
            }
            self.iterations += 1;
        }

        self.get_amoeba_parameter_values();
    }

    /// VTK: `vtkAmoebaMinimizer::Iterate`.
    pub fn iterate(&mut self) -> i32 {
        if self.iterations == 0 {
            if self.function.is_none() {
                return 0;
            }
            self.initialize_amoeba();
        }

        let improved = self.perform_amoeba();
        let mut params_within_tol = false;
        if !improved {
            params_within_tol = self.check_parameter_tolerance();
        }
        self.get_amoeba_parameter_values();
        self.iterations += 1;

        i32::from(improved || !params_within_tol)
    }

    /// VTK: `vtkAmoebaMinimizer::SetFunctionValue`.
    pub fn set_function_value(&mut self, value: f64) {
        if self.function_value != value {
            self.function_value = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetFunctionValue`.
    pub fn get_function_value(&self) -> f64 {
        self.function_value
    }

    /// VTK: `vtkAmoebaMinimizer::SetContractionRatio`.
    pub fn set_contraction_ratio(&mut self, value: f64) {
        let value = value.clamp(0.5, 1.0);
        if self.contraction_ratio != value {
            self.contraction_ratio = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetContractionRatio`.
    pub fn get_contraction_ratio(&self) -> f64 {
        self.contraction_ratio
    }

    /// VTK: `vtkAmoebaMinimizer::SetExpansionRatio`.
    pub fn set_expansion_ratio(&mut self, value: f64) {
        let value = value.clamp(1.0, 2.0);
        if self.expansion_ratio != value {
            self.expansion_ratio = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetExpansionRatio`.
    pub fn get_expansion_ratio(&self) -> f64 {
        self.expansion_ratio
    }

    /// VTK: `vtkAmoebaMinimizer::SetTolerance`.
    pub fn set_tolerance(&mut self, value: f64) {
        if self.tolerance != value {
            self.tolerance = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.tolerance
    }

    /// VTK: `vtkAmoebaMinimizer::SetParameterTolerance`.
    pub fn set_parameter_tolerance(&mut self, value: f64) {
        if self.parameter_tolerance != value {
            self.parameter_tolerance = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetParameterTolerance`.
    pub fn get_parameter_tolerance(&self) -> f64 {
        self.parameter_tolerance
    }

    /// VTK: `vtkAmoebaMinimizer::SetMaxIterations`.
    pub fn set_max_iterations(&mut self, value: i32) {
        if self.max_iterations != value {
            self.max_iterations = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkAmoebaMinimizer::GetMaxIterations`.
    pub fn get_max_iterations(&self) -> i32 {
        self.max_iterations
    }

    /// VTK: `vtkAmoebaMinimizer::GetIterations`.
    pub fn get_iterations(&self) -> i32 {
        self.iterations
    }

    /// VTK: `vtkAmoebaMinimizer::GetFunctionEvaluations`.
    pub fn get_function_evaluations(&self) -> i32 {
        self.function_evaluations
    }

    /// VTK: `vtkAmoebaMinimizer::EvaluateFunction`.
    pub fn evaluate_function(&mut self) {
        if let Some(function) = self.function {
            function(self.function_arg);
        }
        self.function_evaluations += 1;
    }

    fn initialize_amoeba(&mut self) {
        self.terminate_amoeba();

        let n_parameters = self.parameter_values.len();
        self.amoeba_n_steps_no_improvement = 0;
        self.amoeba_vertices = vec![vec![0.0; n_parameters]; n_parameters + 1];
        self.amoeba_values = vec![0.0; n_parameters + 1];
        self.amoeba_sum = vec![0.0; n_parameters];

        for i in 0..=n_parameters {
            for j in 0..n_parameters {
                self.amoeba_vertices[i][j] = self.parameter_values[j];
                if i > 0 && j == i - 1 {
                    self.amoeba_vertices[i][j] =
                        self.parameter_values[j] + self.parameter_scales[j];
                }
                self.amoeba_sum[j] += self.parameter_values[j];
            }
        }

        for i in 0..=n_parameters {
            self.parameter_values[..n_parameters].copy_from_slice(&self.amoeba_vertices[i]);
            self.evaluate_function();
            self.amoeba_values[i] = self.function_value;
        }

        if n_parameters > 0 {
            self.parameter_values[..n_parameters].copy_from_slice(&self.amoeba_vertices[0]);
        }
    }

    fn get_amoeba_parameter_values(&mut self) {
        if self.amoeba_values.is_empty() {
            return;
        }

        let mut low = 0;
        for i in 1..self.amoeba_values.len() {
            if self.amoeba_values[i] < self.amoeba_values[low] {
                low = i;
            }
        }

        if !self.amoeba_vertices.is_empty() {
            let n_parameters = self.parameter_values.len();
            self.parameter_values[..n_parameters].copy_from_slice(&self.amoeba_vertices[low]);
        }
        self.function_value = self.amoeba_values[low];
    }

    fn terminate_amoeba(&mut self) {
        self.amoeba_vertices.clear();
        self.amoeba_values.clear();
        self.amoeba_sum.clear();
    }

    fn try_amoeba(&mut self, high: usize, fac: f64) -> f64 {
        let n_parameters = self.parameter_values.len();
        if n_parameters == 0 {
            return self.function_value;
        }

        let fac1 = (1.0 - fac) / n_parameters as f64;
        let fac2 = fac - fac1;

        for j in 0..n_parameters {
            self.parameter_values[j] =
                self.amoeba_sum[j] * fac1 + self.amoeba_vertices[high][j] * fac2;
        }

        self.evaluate_function();
        let y_try = self.function_value;

        if y_try < self.amoeba_values[high] {
            self.amoeba_values[high] = y_try;
            for j in 0..n_parameters {
                self.amoeba_sum[j] += self.parameter_values[j] - self.amoeba_vertices[high][j];
                self.amoeba_vertices[high][j] = self.parameter_values[j];
            }
        }

        y_try
    }

    fn perform_amoeba(&mut self) -> bool {
        let n_parameters = self.parameter_values.len();
        if n_parameters == 0 || self.amoeba_values.len() < 2 {
            return false;
        }

        let (mut high, mut next_high) = if self.amoeba_values[0] > self.amoeba_values[1] {
            (0, 1)
        } else {
            (1, 0)
        };
        let mut low = next_high;

        for i in 2..=n_parameters {
            if self.amoeba_values[i] < self.amoeba_values[low] {
                low = i;
            } else if self.amoeba_values[i] > self.amoeba_values[high] {
                next_high = high;
                high = i;
            } else if self.amoeba_values[i] > self.amoeba_values[next_high] {
                next_high = i;
            }
        }

        let mut improvement_found = true;
        if self.amoeba_values[high] == self.amoeba_high_value
            || amoeba_numerically_close(
                self.amoeba_values[low],
                self.amoeba_values[high],
                self.tolerance,
            )
        {
            self.amoeba_n_steps_no_improvement += 1;
            if self.amoeba_n_steps_no_improvement >= N_STEPS_NO_VALUE_IMPROVEMENT {
                improvement_found = false;
            }
        } else {
            self.amoeba_n_steps_no_improvement = 0;
        }

        self.amoeba_high_value = self.amoeba_values[high];

        let mut y_try = self.try_amoeba(high, -1.0);
        if y_try <= self.amoeba_values[low] {
            self.try_amoeba(high, self.expansion_ratio);
        } else if y_try >= self.amoeba_values[next_high] {
            let y_save = self.amoeba_values[high];
            y_try = self.try_amoeba(high, self.contraction_ratio);

            if y_try >= y_save {
                for i in 0..=n_parameters {
                    if i != low {
                        for j in 0..n_parameters {
                            self.parameter_values[j] =
                                (self.amoeba_vertices[i][j] + self.amoeba_vertices[low][j]) / 2.0;
                            self.amoeba_vertices[i][j] = self.parameter_values[j];
                        }

                        self.evaluate_function();
                        self.amoeba_values[i] = self.function_value;
                    }
                }

                for j in 0..n_parameters {
                    self.amoeba_sum[j] = 0.0;
                    for i in 0..=n_parameters {
                        self.amoeba_sum[j] += self.amoeba_vertices[i][j];
                    }
                }
            }
        }

        improvement_found
    }

    fn check_parameter_tolerance(&mut self) -> bool {
        let n_parameters = self.parameter_values.len();
        if n_parameters == 0 || self.amoeba_vertices.len() <= n_parameters {
            return true;
        }

        let vertex0 = &self.amoeba_vertices[0];
        let mut size = 0.0;

        for i in 1..=n_parameters {
            for (j, vertex0_value) in vertex0.iter().enumerate().take(n_parameters) {
                let d =
                    ((self.amoeba_vertices[i][j] - vertex0_value) / self.parameter_scales[j]).abs();
                size = if d < size { size } else { d };
            }
        }

        if size != self.amoeba_size {
            self.amoeba_n_steps_no_improvement = N_STEPS_NO_VALUE_IMPROVEMENT - 1;
        }
        self.amoeba_size = size;

        if self.amoeba_n_steps_no_improvement
            > N_STEPS_NO_VALUE_IMPROVEMENT + N_STEPS_NO_PARAM_IMPROVEMENT
        {
            return true;
        }

        size <= self.parameter_tolerance
    }

    fn delete_function_arg(&mut self) {
        if !self.function_arg.is_null() {
            if let Some(function_arg_delete) = self.function_arg_delete {
                function_arg_delete(self.function_arg);
            }
        }
        self.function_arg = ptr::null_mut();
    }
}

impl Drop for AmoebaMinimizer {
    fn drop(&mut self) {
        self.terminate_amoeba();
        self.delete_function_arg();
        self.function_arg_delete = None;
        self.function = None;
    }
}

impl Default for AmoebaMinimizer {
    fn default() -> Self {
        Self::new()
    }
}

fn callback_id(callback: Option<AmoebaCallback>) -> Option<usize> {
    callback.map(|function| function as usize)
}

fn amoeba_numerically_close(n1: f64, n2: f64, threshold_ratio: f64) -> bool {
    let diff = (n1 - n2).abs();
    let abs_n1 = n1.abs();
    let abs_n2 = n2.abs();

    if abs_n1 < VTK_AMOEBA_SMALLEST || abs_n2 < VTK_AMOEBA_SMALLEST {
        return abs_n1 < threshold_ratio && abs_n2 < threshold_ratio;
    }

    let avg = (n1 + n2) / 2.0;
    if avg == 0.0 {
        return diff <= threshold_ratio;
    }

    diff / avg.abs() <= threshold_ratio
}
