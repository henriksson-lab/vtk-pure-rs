use std::ffi::c_void;

use crate::common::core::object::Object;

use super::function_set::FunctionSetHandle;

/// VTK: `vtkInitialValueProblemSolver::ErrorCodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum InitialValueProblemSolverError {
    OutOfDomain = 1,
    NotInitialized = 2,
    UnexpectedValue = 3,
}

impl InitialValueProblemSolverError {
    pub fn code(self) -> i32 {
        self as i32
    }
}

/// VTK: `vtkInitialValueProblemSolver`.
#[derive(Debug, Clone)]
pub struct InitialValueProblemSolver {
    object: Object,
    function_set: Option<FunctionSetHandle>,
    vals: Vec<f64>,
    derivs: Vec<f64>,
    initialized: bool,
    adaptive: bool,
}

impl InitialValueProblemSolver {
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            function_set: None,
            vals: Vec::new(),
            derivs: Vec::new(),
            initialized: false,
            adaptive: false,
        }
    }

    /// VTK: `vtkInitialValueProblemSolver::SetFunctionSet`.
    pub fn set_function_set(&mut self, function_set: Option<FunctionSetHandle>) {
        let valid = function_set.as_ref().is_none_or(|fset| {
            let fset = fset.borrow();
            fset.get_number_of_functions() == fset.get_number_of_independent_variables() - 1
        });

        self.function_set = if valid { function_set } else { None };
        self.object.modified();
        self.initialize();
    }

    /// VTK: `vtkInitialValueProblemSolver::GetFunctionSet`.
    pub fn get_function_set(&self) -> Option<FunctionSetHandle> {
        self.function_set.clone()
    }

    /// VTK: `vtkInitialValueProblemSolver::IsAdaptive`.
    pub fn is_adaptive(&self) -> bool {
        self.adaptive
    }

    pub(crate) fn set_adaptive(&mut self, adaptive: bool) {
        self.adaptive = adaptive;
    }

    /// VTK: `vtkInitialValueProblemSolver::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\n  Function set: {}\n  Function values: {}\n  Function derivatives: {}\n  Initialized: {}",
            self.object.get_class_name(),
            if self.function_set.is_some() { "Some" } else { "None" },
            self.vals.len(),
            self.derivs.len(),
            if self.initialized { "Yes" } else { "No" }
        )
    }

    /// VTK: `vtkInitialValueProblemSolver::Initialize`.
    pub fn initialize(&mut self) {
        let Some(function_set) = &self.function_set else {
            return;
        };

        let function_set = function_set.borrow();
        self.vals = vec![0.0; function_set.get_number_of_independent_variables() as usize];
        self.derivs = vec![0.0; function_set.get_number_of_functions() as usize];
        self.initialized = true;
    }

    /// VTK: `vtkInitialValueProblemSolver::ComputeNextStep`.
    pub fn compute_next_step(
        &mut self,
        _xprev: &[f64],
        _dxprev: Option<&[f64]>,
        _xnext: &mut [f64],
        _t: f64,
        _del_t: &mut f64,
        _del_t_actual: &mut f64,
        _min_step: f64,
        _max_step: f64,
        _max_error: f64,
        _error: &mut f64,
        _user_data: *mut c_void,
    ) -> i32 {
        0
    }

    pub(crate) fn function_set(&self) -> Option<FunctionSetHandle> {
        self.function_set.clone()
    }

    pub(crate) fn vals(&self) -> &[f64] {
        &self.vals
    }

    pub(crate) fn vals_mut(&mut self) -> &mut [f64] {
        &mut self.vals
    }

    pub(crate) fn derivs_mut(&mut self) -> &mut [f64] {
        &mut self.derivs
    }

    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }
}
