use std::ffi::c_void;

use super::{
    function_set::FunctionSetHandle,
    initial_value_problem_solver::{InitialValueProblemSolver, InitialValueProblemSolverError},
};

const A: [f64; 5] = [1.0 / 5.0, 3.0 / 10.0, 3.0 / 5.0, 1.0, 7.0 / 8.0];
const B: [[f64; 5]; 5] = [
    [1.0 / 5.0, 0.0, 0.0, 0.0, 0.0],
    [3.0 / 40.0, 9.0 / 40.0, 0.0, 0.0, 0.0],
    [3.0 / 10.0, -9.0 / 10.0, 6.0 / 5.0, 0.0, 0.0],
    [-11.0 / 54.0, 5.0 / 2.0, -70.0 / 27.0, 35.0 / 27.0, 0.0],
    [
        1631.0 / 55296.0,
        175.0 / 512.0,
        575.0 / 13824.0,
        44275.0 / 110592.0,
        253.0 / 4096.0,
    ],
];
const C: [f64; 6] = [
    37.0 / 378.0,
    0.0,
    250.0 / 621.0,
    125.0 / 594.0,
    0.0,
    512.0 / 1771.0,
];
const DC: [f64; 6] = [
    37.0 / 378.0 - 2825.0 / 27648.0,
    0.0,
    250.0 / 621.0 - 18575.0 / 48384.0,
    125.0 / 594.0 - 13525.0 / 55296.0,
    -277.0 / 14336.0,
    512.0 / 1771.0 - 1.0 / 4.0,
];

/// VTK: `vtkRungeKutta45`.
#[derive(Debug, Clone)]
pub struct RungeKutta45 {
    superclass: InitialValueProblemSolver,
    next_derivs: [Vec<f64>; 6],
}

impl RungeKutta45 {
    /// VTK: `vtkRungeKutta45::New`.
    pub fn new() -> Self {
        let mut superclass = InitialValueProblemSolver::with_class_name("vtkRungeKutta45");
        superclass.set_adaptive(true);
        Self {
            superclass,
            next_derivs: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
        }
    }

    /// VTK: `vtkRungeKutta45::Initialize`.
    pub fn initialize(&mut self) {
        self.superclass.initialize();
        let Some(function_set) = self.superclass.function_set() else {
            return;
        };
        if !self.superclass.initialized() {
            return;
        }
        let num_derivs = function_set.borrow().get_number_of_functions() as usize;
        for next_derivs in &mut self.next_derivs {
            *next_derivs = vec![0.0; num_derivs];
        }
    }

    /// VTK: `vtkRungeKutta45::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.superclass.print_self()
    }

    /// VTK: `vtkInitialValueProblemSolver::SetFunctionSet`.
    pub fn set_function_set(&mut self, function_set: Option<FunctionSetHandle>) {
        self.superclass.set_function_set(function_set);
        self.initialize();
    }

    /// VTK: `vtkInitialValueProblemSolver::GetFunctionSet`.
    pub fn get_function_set(&self) -> Option<FunctionSetHandle> {
        self.superclass.get_function_set()
    }

    /// VTK: `vtkInitialValueProblemSolver::IsAdaptive`.
    pub fn is_adaptive(&self) -> bool {
        self.superclass.is_adaptive()
    }

    /// VTK: `vtkRungeKutta45::ComputeNextStep`.
    pub fn compute_next_step(
        &mut self,
        xprev: &[f64],
        dxprev: Option<&[f64]>,
        xnext: &mut [f64],
        t: f64,
        del_t: &mut f64,
        del_t_actual: &mut f64,
        mut min_step: f64,
        mut max_step: f64,
        max_error: f64,
        est_err: &mut f64,
        user_data: *mut c_void,
    ) -> i32 {
        *est_err = f64::MAX;

        if min_step < 0.0 {
            min_step = -min_step;
        }
        if max_step < 0.0 {
            max_step = -max_step;
        }

        *del_t_actual = 0.0;
        let mut abs_dt = del_t.abs();
        if ((min_step == abs_dt) && (max_step == abs_dt)) || max_error <= 0.0 {
            return self.compute_a_step(
                xprev,
                dxprev,
                xnext,
                t,
                del_t,
                del_t_actual,
                est_err,
                user_data,
            );
        }
        if min_step > max_step {
            return InitialValueProblemSolverError::UnexpectedValue.code();
        }

        let mut should_break = false;
        while *est_err > max_error {
            let ret_val = self.compute_a_step(
                xprev,
                dxprev,
                xnext,
                t,
                del_t,
                del_t_actual,
                est_err,
                user_data,
            );
            if ret_val != 0 {
                return ret_val;
            }

            abs_dt = del_t.abs();
            if abs_dt == min_step {
                break;
            }

            let err_ratio = *est_err / max_error;
            let tmp = if err_ratio == 0.0 {
                if *del_t < 0.0 {
                    -min_step
                } else {
                    min_step
                }
            } else if err_ratio > 1.0 {
                0.9 * *del_t * err_ratio.powf(-0.25)
            } else {
                0.9 * *del_t * err_ratio.powf(-0.2)
            };
            let tmp_abs = tmp.abs();

            if tmp_abs > max_step {
                *del_t = max_step * *del_t / del_t.abs();
                should_break = true;
            } else if tmp_abs < min_step {
                *del_t = min_step * *del_t / del_t.abs();
                should_break = true;
            } else {
                *del_t = tmp;
            }

            if t + *del_t == t {
                return InitialValueProblemSolverError::UnexpectedValue.code();
            }

            if should_break {
                let ret_val = self.compute_a_step(
                    xprev,
                    dxprev,
                    xnext,
                    t,
                    del_t,
                    del_t_actual,
                    est_err,
                    user_data,
                );
                if ret_val != 0 {
                    return ret_val;
                }
                break;
            }
        }

        0
    }

    /// VTK: `vtkRungeKutta45::ComputeAStep`.
    pub fn compute_a_step(
        &mut self,
        xprev: &[f64],
        dxprev: Option<&[f64]>,
        xnext: &mut [f64],
        t: f64,
        del_t: &mut f64,
        del_t_actual: &mut f64,
        error: &mut f64,
        user_data: *mut c_void,
    ) -> i32 {
        *del_t_actual = 0.0;

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
            self.next_derivs[0][..num_derivs].copy_from_slice(&dxprev[..num_derivs]);
        } else if function_set.borrow_mut().function_values(
            self.superclass.vals(),
            &mut self.next_derivs[0],
            user_data,
        ) == 0
        {
            xnext[..num_derivs].copy_from_slice(&self.superclass.vals()[..num_derivs]);
            return InitialValueProblemSolverError::OutOfDomain.code();
        }

        for i in 1..6 {
            for j in 0..num_derivs {
                let mut sum = 0.0;
                for k in 0..i {
                    sum += B[i - 1][k] * self.next_derivs[k][j];
                }
                self.superclass.vals_mut()[j] = xprev[j] + *del_t * sum;
            }
            self.superclass.vals_mut()[num_vals - 1] = t + *del_t * A[i - 1];

            if function_set.borrow_mut().function_values(
                self.superclass.vals(),
                &mut self.next_derivs[i],
                user_data,
            ) == 0
            {
                xnext[..num_derivs].copy_from_slice(&self.superclass.vals()[..num_derivs]);
                *del_t_actual = *del_t * A[i - 1];
                return InitialValueProblemSolverError::OutOfDomain.code();
            }
        }

        for i in 0..num_derivs {
            let mut sum = 0.0;
            for (j, coefficient) in C.iter().enumerate() {
                sum += coefficient * self.next_derivs[j][i];
            }
            xnext[i] = xprev[i] + *del_t * sum;
        }
        *del_t_actual = *del_t;

        let mut err = 0.0;
        for i in 0..num_derivs {
            let mut sum = 0.0;
            for (j, coefficient) in DC.iter().enumerate() {
                sum += coefficient * self.next_derivs[j][i];
            }
            err += *del_t * sum * *del_t * sum;
        }
        *error = err.sqrt();

        let num_zero = (0..num_derivs).filter(|&i| xnext[i] == xprev[i]).count();
        if num_zero == num_derivs {
            return InitialValueProblemSolverError::UnexpectedValue.code();
        }

        0
    }
}
