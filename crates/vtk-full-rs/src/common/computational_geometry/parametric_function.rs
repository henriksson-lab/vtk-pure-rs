use crate::common::core::{Object, VtkMTimeType};

/// Pure virtual interface from `vtkParametricFunction`.
pub trait ParametricFunctionApi {
    /// Access the embedded `vtkParametricFunction` base storage.
    fn parametric_function(&self) -> &ParametricFunction;

    /// Mutable access to the embedded `vtkParametricFunction` base storage.
    fn parametric_function_mut(&mut self) -> &mut ParametricFunction;

    /// VTK: `vtkParametricFunction::GetDimension`.
    fn get_dimension(&self) -> i32;

    /// VTK: `vtkParametricFunction::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], duvw: &mut [f64; 9]);

    /// VTK: `vtkParametricFunction::EvaluateScalar`.
    fn evaluate_scalar(&self, uvw: [f64; 3], pt: [f64; 3], duvw: [f64; 9]) -> f64;
}

/// Storage and common API for `vtkParametricFunction`.
///
/// VTK `vtkParametricFunction` is abstract. Concrete Rust parametric
/// functions embed this base and implement their own `GetDimension`,
/// `Evaluate`, and `EvaluateScalar` equivalents.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricFunction {
    object: Object,
    minimum_u: f64,
    maximum_u: f64,
    minimum_v: f64,
    maximum_v: f64,
    minimum_w: f64,
    maximum_w: f64,
    join_u: bool,
    join_v: bool,
    join_w: bool,
    twist_u: bool,
    twist_v: bool,
    twist_w: bool,
    clockwise_ordering: bool,
    derivatives_available: bool,
}

impl ParametricFunction {
    /// VTK: protected `vtkParametricFunction::vtkParametricFunction`.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::with_class_name("vtkParametricFunction")
    }

    #[allow(dead_code)]
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            minimum_u: 0.0,
            maximum_u: 1.0,
            minimum_v: 0.0,
            maximum_v: 1.0,
            minimum_w: 0.0,
            maximum_w: 1.0,
            join_u: false,
            join_v: false,
            join_w: false,
            twist_u: false,
            twist_v: false,
            twist_w: false,
            clockwise_ordering: true,
            derivatives_available: true,
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

    /// VTK: `vtkParametricFunction::SetMinimumU`.
    pub fn set_minimum_u(&mut self, value: f64) {
        if self.minimum_u != value {
            self.minimum_u = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetMinimumU`.
    pub fn get_minimum_u(&self) -> f64 {
        self.minimum_u
    }

    /// VTK: `vtkParametricFunction::SetMaximumU`.
    pub fn set_maximum_u(&mut self, value: f64) {
        if self.maximum_u != value {
            self.maximum_u = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetMaximumU`.
    pub fn get_maximum_u(&self) -> f64 {
        self.maximum_u
    }

    /// VTK: `vtkParametricFunction::SetMinimumV`.
    pub fn set_minimum_v(&mut self, value: f64) {
        if self.minimum_v != value {
            self.minimum_v = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetMinimumV`.
    pub fn get_minimum_v(&self) -> f64 {
        self.minimum_v
    }

    /// VTK: `vtkParametricFunction::SetMaximumV`.
    pub fn set_maximum_v(&mut self, value: f64) {
        if self.maximum_v != value {
            self.maximum_v = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetMaximumV`.
    pub fn get_maximum_v(&self) -> f64 {
        self.maximum_v
    }

    /// VTK: `vtkParametricFunction::SetMinimumW`.
    pub fn set_minimum_w(&mut self, value: f64) {
        if self.minimum_w != value {
            self.minimum_w = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetMinimumW`.
    pub fn get_minimum_w(&self) -> f64 {
        self.minimum_w
    }

    /// VTK: `vtkParametricFunction::SetMaximumW`.
    pub fn set_maximum_w(&mut self, value: f64) {
        if self.maximum_w != value {
            self.maximum_w = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetMaximumW`.
    pub fn get_maximum_w(&self) -> f64 {
        self.maximum_w
    }

    /// VTK: `vtkParametricFunction::SetJoinU`.
    pub fn set_join_u(&mut self, value: bool) {
        if self.join_u != value {
            self.join_u = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetJoinU`.
    pub fn get_join_u(&self) -> bool {
        self.join_u
    }

    /// VTK: `vtkParametricFunction::JoinUOn`.
    pub fn join_u_on(&mut self) {
        self.set_join_u(true);
    }

    /// VTK: `vtkParametricFunction::JoinUOff`.
    pub fn join_u_off(&mut self) {
        self.set_join_u(false);
    }

    /// VTK: `vtkParametricFunction::SetJoinV`.
    pub fn set_join_v(&mut self, value: bool) {
        if self.join_v != value {
            self.join_v = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetJoinV`.
    pub fn get_join_v(&self) -> bool {
        self.join_v
    }

    /// VTK: `vtkParametricFunction::JoinVOn`.
    pub fn join_v_on(&mut self) {
        self.set_join_v(true);
    }

    /// VTK: `vtkParametricFunction::JoinVOff`.
    pub fn join_v_off(&mut self) {
        self.set_join_v(false);
    }

    /// VTK: `vtkParametricFunction::SetJoinW`.
    pub fn set_join_w(&mut self, value: bool) {
        if self.join_w != value {
            self.join_w = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetJoinW`.
    pub fn get_join_w(&self) -> bool {
        self.join_w
    }

    /// VTK: `vtkParametricFunction::JoinWOn`.
    pub fn join_w_on(&mut self) {
        self.set_join_w(true);
    }

    /// VTK: `vtkParametricFunction::JoinWOff`.
    pub fn join_w_off(&mut self) {
        self.set_join_w(false);
    }

    /// VTK: `vtkParametricFunction::SetTwistU`.
    pub fn set_twist_u(&mut self, value: bool) {
        if self.twist_u != value {
            self.twist_u = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetTwistU`.
    pub fn get_twist_u(&self) -> bool {
        self.twist_u
    }

    /// VTK: `vtkParametricFunction::TwistUOn`.
    pub fn twist_u_on(&mut self) {
        self.set_twist_u(true);
    }

    /// VTK: `vtkParametricFunction::TwistUOff`.
    pub fn twist_u_off(&mut self) {
        self.set_twist_u(false);
    }

    /// VTK: `vtkParametricFunction::SetTwistV`.
    pub fn set_twist_v(&mut self, value: bool) {
        if self.twist_v != value {
            self.twist_v = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetTwistV`.
    pub fn get_twist_v(&self) -> bool {
        self.twist_v
    }

    /// VTK: `vtkParametricFunction::TwistVOn`.
    pub fn twist_v_on(&mut self) {
        self.set_twist_v(true);
    }

    /// VTK: `vtkParametricFunction::TwistVOff`.
    pub fn twist_v_off(&mut self) {
        self.set_twist_v(false);
    }

    /// VTK: `vtkParametricFunction::SetTwistW`.
    pub fn set_twist_w(&mut self, value: bool) {
        if self.twist_w != value {
            self.twist_w = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetTwistW`.
    pub fn get_twist_w(&self) -> bool {
        self.twist_w
    }

    /// VTK: `vtkParametricFunction::TwistWOn`.
    pub fn twist_w_on(&mut self) {
        self.set_twist_w(true);
    }

    /// VTK: `vtkParametricFunction::TwistWOff`.
    pub fn twist_w_off(&mut self) {
        self.set_twist_w(false);
    }

    /// VTK: `vtkParametricFunction::SetClockwiseOrdering`.
    pub fn set_clockwise_ordering(&mut self, value: bool) {
        if self.clockwise_ordering != value {
            self.clockwise_ordering = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetClockwiseOrdering`.
    pub fn get_clockwise_ordering(&self) -> bool {
        self.clockwise_ordering
    }

    /// VTK: `vtkParametricFunction::ClockwiseOrderingOn`.
    pub fn clockwise_ordering_on(&mut self) {
        self.set_clockwise_ordering(true);
    }

    /// VTK: `vtkParametricFunction::ClockwiseOrderingOff`.
    pub fn clockwise_ordering_off(&mut self) {
        self.set_clockwise_ordering(false);
    }

    /// VTK: `vtkParametricFunction::SetDerivativesAvailable`.
    pub fn set_derivatives_available(&mut self, value: bool) {
        if self.derivatives_available != value {
            self.derivatives_available = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricFunction::GetDerivativesAvailable`.
    pub fn get_derivatives_available(&self) -> bool {
        self.derivatives_available
    }

    /// VTK: `vtkParametricFunction::DerivativesAvailableOn`.
    pub fn derivatives_available_on(&mut self) {
        self.set_derivatives_available(true);
    }

    /// VTK: `vtkParametricFunction::DerivativesAvailableOff`.
    pub fn derivatives_available_off(&mut self) {
        self.set_derivatives_available(false);
    }
}
