use std::ffi::c_void;

use super::{
    function_set::FunctionSetHandle,
    initial_value_problem_solver::{InitialValueProblemSolver, InitialValueProblemSolverError},
};

/// VTK: `vtkRungeKutta2`.
#[derive(Debug, Clone)]
pub struct RungeKutta2 {
    superclass: InitialValueProblemSolver,
}

impl RungeKutta2 {
    /// VTK: `vtkRungeKutta2::New`.
    pub fn new() -> Self {
        Self {
            superclass: InitialValueProblemSolver::with_class_name("vtkRungeKutta2"),
        }
    }

    /// VTK: `vtkRungeKutta2::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.superclass.print_self()
    }

    /// VTK: `vtkInitialValueProblemSolver::SetFunctionSet`.
    pub fn set_function_set(&mut self, function_set: Option<FunctionSetHandle>) {
        self.superclass.set_function_set(function_set);
    }

    /// VTK: `vtkInitialValueProblemSolver::GetFunctionSet`.
    pub fn get_function_set(&self) -> Option<FunctionSetHandle> {
        self.superclass.get_function_set()
    }

    /// VTK: `vtkInitialValueProblemSolver::IsAdaptive`.
    pub fn is_adaptive(&self) -> bool {
        self.superclass.is_adaptive()
    }

    /// VTK: `vtkRungeKutta2::ComputeNextStep`.
    pub fn compute_next_step(
        &mut self,
        xprev: &[f64],
        dxprev: Option<&[f64]>,
        xnext: &mut [f64],
        t: f64,
        del_t: &mut f64,
        del_t_actual: &mut f64,
        _min_step: f64,
        _max_step: f64,
        _max_error: f64,
        error: &mut f64,
        user_data: *mut c_void,
    ) -> i32 {
        *del_t_actual = 0.0;
        *error = 0.0;

        let Some(function_set) = self.superclass.function_set() else {
            return InitialValueProblemSolverError::NotInitialized.code();
        };
        if !self.superclass.initialized() {
            return InitialValueProblemSolverError::NotInitialized.code();
        }

        let num_derivs = function_set.borrow().get_number_of_functions() as usize;
        let num_vals = num_derivs + 1;
        if xprev.len() < num_derivs || xnext.len() < num_derivs {
            return InitialValueProblemSolverError::UnexpectedValue.code();
        }

        self.superclass.vals_mut()[..num_derivs].copy_from_slice(&xprev[..num_derivs]);
        self.superclass.vals_mut()[num_vals - 1] = t;

        if let Some(dxprev) = dxprev {
            if dxprev.len() < num_derivs {
                return InitialValueProblemSolverError::UnexpectedValue.code();
            }
            self.superclass.derivs_mut()[..num_derivs].copy_from_slice(&dxprev[..num_derivs]);
        } else if {
            let vals = self.superclass.vals().to_vec();
            function_set.borrow_mut().function_values(
                &vals,
                self.superclass.derivs_mut(),
                user_data,
            ) == 0
        } {
            xnext[..num_derivs].copy_from_slice(&self.superclass.vals()[..num_derivs]);
            return InitialValueProblemSolverError::OutOfDomain.code();
        }

        let derivs = self.superclass.derivs_mut().to_vec();
        for i in 0..num_derivs {
            self.superclass.vals_mut()[i] = xprev[i] + *del_t / 2.0 * derivs[i];
        }
        self.superclass.vals_mut()[num_vals - 1] = t + *del_t / 2.0;

        if {
            let vals = self.superclass.vals().to_vec();
            function_set.borrow_mut().function_values(
                &vals,
                self.superclass.derivs_mut(),
                user_data,
            ) == 0
        } {
            xnext[..num_derivs].copy_from_slice(&self.superclass.vals()[..num_derivs]);
            *del_t_actual = *del_t / 2.0;
            return InitialValueProblemSolverError::OutOfDomain.code();
        }

        let derivs = self.superclass.derivs_mut().to_vec();
        for i in 0..num_derivs {
            xnext[i] = xprev[i] + *del_t * derivs[i];
        }

        *del_t_actual = *del_t;
        0
    }
}
