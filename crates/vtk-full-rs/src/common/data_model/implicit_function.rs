use std::{cell::RefCell, rc::Rc};

use crate::common::core::VtkMTimeType;
use crate::common::data_model::{
    Cone, CoordinateFrame, Cylinder, Frustum, ImplicitBoolean, ImplicitHalo, ImplicitSum,
    ImplicitWindowFunction, PerlinNoise, Plane, Planes, Quadric, Sphere, Spheres, Superquadric,
};

/// VTK virtual API for `vtkImplicitFunction`.
pub trait ImplicitFunctionApi {
    /// VTK: `vtkImplicitFunction::EvaluateFunction`.
    fn evaluate_function(&self, x: [f64; 3]) -> f64;

    /// VTK: `vtkImplicitFunction::EvaluateGradient`.
    fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3];

    /// VTK: `vtkObject::GetMTime`.
    fn get_m_time(&self) -> VtkMTimeType;

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str;
}

/// Rust equivalent of a `vtkImplicitFunction*` reference.
#[derive(Clone)]
pub struct ImplicitFunctionHandle(Rc<RefCell<dyn ImplicitFunctionApi>>);

impl ImplicitFunctionHandle {
    pub fn new<T: ImplicitFunctionApi + 'static>(function: T) -> Self {
        Self(Rc::new(RefCell::new(function)))
    }

    pub fn from_rc<T: ImplicitFunctionApi + 'static>(function: Rc<RefCell<T>>) -> Self {
        Self(function)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        self.0.borrow().evaluate_function(x)
    }

    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        self.0.borrow().evaluate_gradient(x)
    }

    pub fn get_m_time(&self) -> VtkMTimeType {
        self.0.borrow().get_m_time()
    }

    pub fn get_class_name(&self) -> &'static str {
        self.0.borrow().get_class_name()
    }
}

impl<T: ImplicitFunctionApi + 'static> From<Rc<RefCell<T>>> for ImplicitFunctionHandle {
    fn from(function: Rc<RefCell<T>>) -> Self {
        Self::from_rc(function)
    }
}

macro_rules! impl_implicit_function_api_returning_gradient {
    ($type:ty) => {
        impl ImplicitFunctionApi for $type {
            fn evaluate_function(&self, x: [f64; 3]) -> f64 {
                <$type>::evaluate_function(self, x)
            }

            fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
                <$type>::evaluate_gradient(self, x)
            }

            fn get_m_time(&self) -> VtkMTimeType {
                <$type>::get_m_time(self)
            }

            fn get_class_name(&self) -> &'static str {
                <$type>::get_class_name(self)
            }
        }
    };
}

macro_rules! impl_implicit_function_api_mut_gradient {
    ($type:ty) => {
        impl ImplicitFunctionApi for $type {
            fn evaluate_function(&self, x: [f64; 3]) -> f64 {
                <$type>::evaluate_function(self, x)
            }

            fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
                let mut gradient = [0.0; 3];
                <$type>::evaluate_gradient(self, x, &mut gradient);
                gradient
            }

            fn get_m_time(&self) -> VtkMTimeType {
                <$type>::get_m_time(self)
            }

            fn get_class_name(&self) -> &'static str {
                <$type>::get_class_name(self)
            }
        }
    };
}

impl_implicit_function_api_returning_gradient!(Cone);
impl_implicit_function_api_returning_gradient!(CoordinateFrame);
impl_implicit_function_api_returning_gradient!(Frustum);
impl_implicit_function_api_returning_gradient!(ImplicitBoolean);
impl_implicit_function_api_returning_gradient!(ImplicitHalo);
impl_implicit_function_api_returning_gradient!(ImplicitSum);
impl_implicit_function_api_returning_gradient!(ImplicitWindowFunction);
impl_implicit_function_api_returning_gradient!(PerlinNoise);
impl_implicit_function_api_returning_gradient!(Plane);
impl_implicit_function_api_returning_gradient!(Sphere);
impl_implicit_function_api_returning_gradient!(Superquadric);

impl_implicit_function_api_mut_gradient!(Cylinder);
impl_implicit_function_api_mut_gradient!(Planes);
impl_implicit_function_api_mut_gradient!(Quadric);
impl_implicit_function_api_mut_gradient!(Spheres);
