use crate::common::{
    computational_geometry::{ParametricFunction, ParametricFunctionApi},
    core::{math::pi, VtkMTimeType},
};

/// VTK: `vtkParametricDini`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricDini {
    base: ParametricFunction,
    a: f64,
    b: f64,
}

impl ParametricDini {
    /// VTK: `vtkParametricDini::New`.
    pub fn new() -> Self {
        let mut base = ParametricFunction::with_class_name("vtkParametricDini");
        base.set_minimum_u(0.0);
        base.set_maximum_u(4.0 * pi());
        base.set_minimum_v(0.001);
        base.set_maximum_v(2.0);
        base.set_join_u(false);
        base.set_join_v(false);
        base.set_twist_u(false);
        base.set_twist_v(false);
        base.set_clockwise_ordering(false);
        base.set_derivatives_available(true);

        Self {
            base,
            a: 1.0,
            b: 0.2,
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.base.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.base.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.base.get_m_time()
    }

    /// VTK: `vtkParametricDini::SetA`.
    pub fn set_a(&mut self, value: f64) {
        if self.a != value {
            self.a = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricDini::GetA`.
    pub fn get_a(&self) -> f64 {
        self.a
    }

    /// VTK: `vtkParametricDini::SetB`.
    pub fn set_b(&mut self, value: f64) {
        if self.b != value {
            self.b = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricDini::GetB`.
    pub fn get_b(&self) -> f64 {
        self.b
    }

    /// VTK: `vtkParametricFunction::SetMinimumU`.
    pub fn set_minimum_u(&mut self, value: f64) {
        self.base.set_minimum_u(value);
    }

    /// VTK: `vtkParametricFunction::GetMinimumU`.
    pub fn get_minimum_u(&self) -> f64 {
        self.base.get_minimum_u()
    }

    /// VTK: `vtkParametricFunction::SetMaximumU`.
    pub fn set_maximum_u(&mut self, value: f64) {
        self.base.set_maximum_u(value);
    }

    /// VTK: `vtkParametricFunction::GetMaximumU`.
    pub fn get_maximum_u(&self) -> f64 {
        self.base.get_maximum_u()
    }

    /// VTK: `vtkParametricFunction::SetMinimumV`.
    pub fn set_minimum_v(&mut self, value: f64) {
        self.base.set_minimum_v(value);
    }

    /// VTK: `vtkParametricFunction::GetMinimumV`.
    pub fn get_minimum_v(&self) -> f64 {
        self.base.get_minimum_v()
    }

    /// VTK: `vtkParametricFunction::SetMaximumV`.
    pub fn set_maximum_v(&mut self, value: f64) {
        self.base.set_maximum_v(value);
    }

    /// VTK: `vtkParametricFunction::GetMaximumV`.
    pub fn get_maximum_v(&self) -> f64 {
        self.base.get_maximum_v()
    }

    /// VTK: `vtkParametricFunction::SetMinimumW`.
    pub fn set_minimum_w(&mut self, value: f64) {
        self.base.set_minimum_w(value);
    }

    /// VTK: `vtkParametricFunction::GetMinimumW`.
    pub fn get_minimum_w(&self) -> f64 {
        self.base.get_minimum_w()
    }

    /// VTK: `vtkParametricFunction::SetMaximumW`.
    pub fn set_maximum_w(&mut self, value: f64) {
        self.base.set_maximum_w(value);
    }

    /// VTK: `vtkParametricFunction::GetMaximumW`.
    pub fn get_maximum_w(&self) -> f64 {
        self.base.get_maximum_w()
    }

    /// VTK: `vtkParametricFunction::SetJoinU`.
    pub fn set_join_u(&mut self, value: bool) {
        self.base.set_join_u(value);
    }

    /// VTK: `vtkParametricFunction::GetJoinU`.
    pub fn get_join_u(&self) -> bool {
        self.base.get_join_u()
    }

    /// VTK: `vtkParametricFunction::JoinUOn`.
    pub fn join_u_on(&mut self) {
        self.base.join_u_on();
    }

    /// VTK: `vtkParametricFunction::JoinUOff`.
    pub fn join_u_off(&mut self) {
        self.base.join_u_off();
    }

    /// VTK: `vtkParametricFunction::SetJoinV`.
    pub fn set_join_v(&mut self, value: bool) {
        self.base.set_join_v(value);
    }

    /// VTK: `vtkParametricFunction::GetJoinV`.
    pub fn get_join_v(&self) -> bool {
        self.base.get_join_v()
    }

    /// VTK: `vtkParametricFunction::JoinVOn`.
    pub fn join_v_on(&mut self) {
        self.base.join_v_on();
    }

    /// VTK: `vtkParametricFunction::JoinVOff`.
    pub fn join_v_off(&mut self) {
        self.base.join_v_off();
    }

    /// VTK: `vtkParametricFunction::SetJoinW`.
    pub fn set_join_w(&mut self, value: bool) {
        self.base.set_join_w(value);
    }

    /// VTK: `vtkParametricFunction::GetJoinW`.
    pub fn get_join_w(&self) -> bool {
        self.base.get_join_w()
    }

    /// VTK: `vtkParametricFunction::JoinWOn`.
    pub fn join_w_on(&mut self) {
        self.base.join_w_on();
    }

    /// VTK: `vtkParametricFunction::JoinWOff`.
    pub fn join_w_off(&mut self) {
        self.base.join_w_off();
    }

    /// VTK: `vtkParametricFunction::SetTwistU`.
    pub fn set_twist_u(&mut self, value: bool) {
        self.base.set_twist_u(value);
    }

    /// VTK: `vtkParametricFunction::GetTwistU`.
    pub fn get_twist_u(&self) -> bool {
        self.base.get_twist_u()
    }

    /// VTK: `vtkParametricFunction::TwistUOn`.
    pub fn twist_u_on(&mut self) {
        self.base.twist_u_on();
    }

    /// VTK: `vtkParametricFunction::TwistUOff`.
    pub fn twist_u_off(&mut self) {
        self.base.twist_u_off();
    }

    /// VTK: `vtkParametricFunction::SetTwistV`.
    pub fn set_twist_v(&mut self, value: bool) {
        self.base.set_twist_v(value);
    }

    /// VTK: `vtkParametricFunction::GetTwistV`.
    pub fn get_twist_v(&self) -> bool {
        self.base.get_twist_v()
    }

    /// VTK: `vtkParametricFunction::TwistVOn`.
    pub fn twist_v_on(&mut self) {
        self.base.twist_v_on();
    }

    /// VTK: `vtkParametricFunction::TwistVOff`.
    pub fn twist_v_off(&mut self) {
        self.base.twist_v_off();
    }

    /// VTK: `vtkParametricFunction::SetTwistW`.
    pub fn set_twist_w(&mut self, value: bool) {
        self.base.set_twist_w(value);
    }

    /// VTK: `vtkParametricFunction::GetTwistW`.
    pub fn get_twist_w(&self) -> bool {
        self.base.get_twist_w()
    }

    /// VTK: `vtkParametricFunction::TwistWOn`.
    pub fn twist_w_on(&mut self) {
        self.base.twist_w_on();
    }

    /// VTK: `vtkParametricFunction::TwistWOff`.
    pub fn twist_w_off(&mut self) {
        self.base.twist_w_off();
    }

    /// VTK: `vtkParametricFunction::SetClockwiseOrdering`.
    pub fn set_clockwise_ordering(&mut self, value: bool) {
        self.base.set_clockwise_ordering(value);
    }

    /// VTK: `vtkParametricFunction::GetClockwiseOrdering`.
    pub fn get_clockwise_ordering(&self) -> bool {
        self.base.get_clockwise_ordering()
    }

    /// VTK: `vtkParametricFunction::ClockwiseOrderingOn`.
    pub fn clockwise_ordering_on(&mut self) {
        self.base.clockwise_ordering_on();
    }

    /// VTK: `vtkParametricFunction::ClockwiseOrderingOff`.
    pub fn clockwise_ordering_off(&mut self) {
        self.base.clockwise_ordering_off();
    }

    /// VTK: `vtkParametricFunction::SetDerivativesAvailable`.
    pub fn set_derivatives_available(&mut self, value: bool) {
        self.base.set_derivatives_available(value);
    }

    /// VTK: `vtkParametricFunction::GetDerivativesAvailable`.
    pub fn get_derivatives_available(&self) -> bool {
        self.base.get_derivatives_available()
    }

    /// VTK: `vtkParametricFunction::DerivativesAvailableOn`.
    pub fn derivatives_available_on(&mut self) {
        self.base.derivatives_available_on();
    }

    /// VTK: `vtkParametricFunction::DerivativesAvailableOff`.
    pub fn derivatives_available_off(&mut self) {
        self.base.derivatives_available_off();
    }
}

impl ParametricFunctionApi for ParametricDini {
    fn parametric_function(&self) -> &ParametricFunction {
        &self.base
    }

    fn parametric_function_mut(&mut self) -> &mut ParametricFunction {
        &mut self.base
    }

    /// VTK: `vtkParametricDini::GetDimension`.
    fn get_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkParametricDini::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], duvw: &mut [f64; 9]) {
        let u = uvw[0];
        let v = uvw[1];
        let cu = u.cos();
        let cv = v.cos();
        let su = u.sin();
        let sv = v.sin();

        pt[0] = self.a * cu * sv;
        pt[1] = self.a * su * sv;
        let tolerance = 0.0001;
        if v.abs() > tolerance {
            pt[2] = self.a * (v.cos() + (v / 2.0).tan().ln()) + self.b * u;
        } else {
            pt[2] = self.a * (v.cos() + (tolerance / 2.0).tan().ln()) + self.b * u;
        }

        duvw[0] = -pt[1];
        duvw[3] = self.a * cu * cv;
        duvw[1] = pt[0];
        duvw[4] = self.a * su * cv;
        duvw[2] = self.b;
        let tv2 = (0.5 * v).tan();
        if tv2 != 0.0 {
            duvw[5] = self.a * (-sv + (0.5 + 0.5 * tv2 * tv2) / tv2);
        } else {
            duvw[5] = self.a * (-sv + 0.00005);
        }
    }

    /// VTK: `vtkParametricDini::EvaluateScalar`.
    fn evaluate_scalar(&self, _uvw: [f64; 3], _pt: [f64; 3], _duvw: [f64; 9]) -> f64 {
        0.0
    }
}

impl Default for ParametricDini {
    fn default() -> Self {
        Self::new()
    }
}
