use std::fmt;

use crate::common::core::{Object, VtkMTimeType, VTK_DOUBLE_MAX};
use crate::common::data_model::{ImplicitFunctionCollection, ImplicitFunctionHandle};

pub const VTK_UNION: i32 = 0;
pub const VTK_INTERSECTION: i32 = 1;
pub const VTK_DIFFERENCE: i32 = 2;
pub const VTK_UNION_OF_MAGNITUDES: i32 = 3;

/// VTK: `vtkImplicitBoolean`.
#[derive(Clone)]
pub struct ImplicitBoolean {
    object: Object,
    function_list: ImplicitFunctionCollection,
    operation_type: i32,
}

impl ImplicitBoolean {
    /// VTK: `vtkImplicitBoolean::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkImplicitBoolean"),
            function_list: ImplicitFunctionCollection::new(),
            operation_type: VTK_UNION,
        }
    }

    /// VTK: `vtkImplicitBoolean::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.function_list
            .iter()
            .fold(self.object.get_m_time(), |mtime, function| {
                mtime.max(function.get_m_time())
            })
    }

    /// VTK: `vtkImplicitBoolean::AddFunction`.
    pub fn add_function(&mut self, function: ImplicitFunctionHandle) {
        if self.function_list.index_of_first_occurrence(&function) >= 0 {
            return;
        }
        self.modified();
        self.function_list.add_item(function);
    }

    /// VTK: `vtkImplicitBoolean::RemoveFunction`.
    pub fn remove_function(&mut self, function: &ImplicitFunctionHandle) {
        let old_len = self.function_list.len();
        self.function_list.remove_item(function);
        if self.function_list.len() != old_len {
            self.modified();
        }
    }

    /// VTK: `vtkImplicitBoolean::GetFunction`.
    pub fn get_function(&self) -> &ImplicitFunctionCollection {
        &self.function_list
    }

    /// VTK: `vtkImplicitBoolean::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        if self.function_list.is_empty() {
            return 0.0;
        }

        match self.operation_type {
            VTK_UNION => self
                .function_list
                .iter()
                .fold(VTK_DOUBLE_MAX, |value, function| {
                    value.min(function.evaluate_function(x))
                }),
            VTK_INTERSECTION => self
                .function_list
                .iter()
                .fold(-VTK_DOUBLE_MAX, |value, function| {
                    value.max(function.evaluate_function(x))
                }),
            VTK_UNION_OF_MAGNITUDES => self
                .function_list
                .iter()
                .fold(VTK_DOUBLE_MAX, |value, function| {
                    value.min(function.evaluate_function(x).abs())
                }),
            _ => {
                let first = self
                    .function_list
                    .first()
                    .expect("non-empty implicit function collection");
                let mut value = first.evaluate_function(x);
                for function in self.function_list.iter() {
                    if !function.ptr_eq(first) {
                        value = value.max(-function.evaluate_function(x));
                    }
                }
                value
            }
        }
    }

    /// VTK: `vtkImplicitBoolean::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        if self.function_list.is_empty() {
            return [0.0; 3];
        }

        if self.operation_type == VTK_UNION_OF_MAGNITUDES {
            let mut value = VTK_DOUBLE_MAX;
            let mut gradient = [0.0; 3];
            for function in self.function_list.iter() {
                let v = function.evaluate_function(x).abs();
                if v < value {
                    value = v;
                    gradient = function.evaluate_gradient(x);
                }
            }
            return gradient;
        }

        let first = self
            .function_list
            .first()
            .expect("non-empty implicit function collection");
        let mut value = first.evaluate_function(x);
        let mut gradient = negate(first.evaluate_gradient(x));
        for function in self.function_list.iter() {
            if !function.ptr_eq(first) {
                let v = -function.evaluate_function(x);
                if v > value {
                    value = v;
                    gradient = negate(function.evaluate_gradient(x));
                }
            }
        }
        gradient
    }

    /// VTK: `vtkImplicitBoolean::SetOperationType`.
    pub fn set_operation_type(&mut self, operation_type: i32) {
        let operation_type = operation_type.clamp(VTK_UNION, VTK_UNION_OF_MAGNITUDES);
        if self.operation_type != operation_type {
            self.operation_type = operation_type;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitBoolean::GetOperationType`.
    pub fn get_operation_type(&self) -> i32 {
        self.operation_type
    }

    /// VTK: `vtkImplicitBoolean::SetOperationTypeToUnion`.
    pub fn set_operation_type_to_union(&mut self) {
        self.set_operation_type(VTK_UNION);
    }

    /// VTK: `vtkImplicitBoolean::SetOperationTypeToIntersection`.
    pub fn set_operation_type_to_intersection(&mut self) {
        self.set_operation_type(VTK_INTERSECTION);
    }

    /// VTK: `vtkImplicitBoolean::SetOperationTypeToDifference`.
    pub fn set_operation_type_to_difference(&mut self) {
        self.set_operation_type(VTK_DIFFERENCE);
    }

    /// VTK: `vtkImplicitBoolean::SetOperationTypeToUnionOfMagnitudes`.
    pub fn set_operation_type_to_union_of_magnitudes(&mut self) {
        self.set_operation_type(VTK_UNION_OF_MAGNITUDES);
    }

    /// VTK: `vtkImplicitBoolean::GetOperationTypeAsString`.
    pub fn get_operation_type_as_string(&self) -> &'static str {
        match self.operation_type {
            VTK_UNION => "Union",
            VTK_INTERSECTION => "Intersection",
            VTK_DIFFERENCE => "Difference",
            _ => "UnionOfMagnitudes",
        }
    }

    /// VTK: `vtkImplicitBoolean::PrintSelf`.
    pub fn print_self(&self) -> String {
        let operation = match self.operation_type {
            VTK_INTERSECTION => "VTK_INTERSECTION",
            VTK_UNION => "VTK_UNION",
            VTK_UNION_OF_MAGNITUDES => "VTK_UNION_OF_MAGNITUDES",
            _ => "VTK_DIFFERENCE",
        };
        format!(
            "Function List: {} items\nOperator Type: {}\n",
            self.function_list.len(),
            operation
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
}

impl Default for ImplicitBoolean {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ImplicitBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImplicitBoolean")
            .field("class_name", &self.get_class_name())
            .field("function_count", &self.function_list.len())
            .field("operation_type", &self.operation_type)
            .finish()
    }
}

fn negate(gradient: [f64; 3]) -> [f64; 3] {
    [-gradient[0], -gradient[1], -gradient[2]]
}
