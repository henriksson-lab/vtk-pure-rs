use std::{cell::RefCell, ffi::c_void, fmt, rc::Rc};

use crate::common::core::object::Object;

pub type FunctionSetHandle = Rc<RefCell<dyn FunctionSetApi>>;

/// VTK: `vtkFunctionSet`.
pub trait FunctionSetApi: fmt::Debug {
    /// VTK: `vtkFunctionSet::FunctionValues`.
    fn function_values(&mut self, x: &[f64], f: &mut [f64], user_data: *mut c_void) -> i32;

    /// VTK: `vtkFunctionSet::GetNumberOfFunctions`.
    fn get_number_of_functions(&self) -> i32;

    /// VTK: `vtkFunctionSet::GetNumberOfIndependentVariables`.
    fn get_number_of_independent_variables(&self) -> i32;
}

/// VTK: `vtkFunctionSet`.
#[derive(Debug, Clone)]
pub struct FunctionSet {
    object: Object,
    num_funcs: i32,
    num_indep_vars: i32,
}

impl FunctionSet {
    /// VTK: `vtkFunctionSet::vtkFunctionSet`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkFunctionSet"),
            num_funcs: 0,
            num_indep_vars: 0,
        }
    }

    /// VTK: `vtkFunctionSet::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\n  Number of functions: {}\n  Number of independent variables: {}",
            self.object.get_class_name(),
            self.num_funcs,
            self.num_indep_vars
        )
    }

    /// VTK: `vtkFunctionSet::FunctionValues`.
    pub fn function_values(&mut self, x: &[f64], f: &mut [f64]) -> i32 {
        self.function_values_with_user_data(x, f, std::ptr::null_mut())
    }

    /// VTK: `vtkFunctionSet::FunctionValues`.
    pub fn function_values_with_user_data(
        &mut self,
        _x: &[f64],
        _f: &mut [f64],
        _user_data: *mut c_void,
    ) -> i32 {
        0
    }

    /// VTK: `vtkFunctionSet::GetNumberOfFunctions`.
    pub fn get_number_of_functions(&self) -> i32 {
        self.num_funcs
    }

    /// VTK: `vtkFunctionSet::GetNumberOfIndependentVariables`.
    pub fn get_number_of_independent_variables(&self) -> i32 {
        self.num_indep_vars
    }
}

impl FunctionSetApi for FunctionSet {
    fn function_values(&mut self, x: &[f64], f: &mut [f64], user_data: *mut c_void) -> i32 {
        self.function_values_with_user_data(x, f, user_data)
    }

    fn get_number_of_functions(&self) -> i32 {
        self.get_number_of_functions()
    }

    fn get_number_of_independent_variables(&self) -> i32 {
        self.get_number_of_independent_variables()
    }
}
