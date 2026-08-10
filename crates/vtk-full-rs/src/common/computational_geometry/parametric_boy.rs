use crate::common::{
    computational_geometry::{ParametricFunction, ParametricFunctionApi},
    core::{math::pi, VtkMTimeType},
};

/// VTK: `vtkParametricBoy`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricBoy {
    base: ParametricFunction,
    z_scale: f64,
}

impl ParametricBoy {
    /// VTK: `vtkParametricBoy::New`.
    pub fn new() -> Self {
        let mut base = ParametricFunction::with_class_name("vtkParametricBoy");
        base.set_minimum_u(0.0);
        base.set_maximum_u(pi());
        base.set_minimum_v(0.0);
        base.set_maximum_v(pi());
        base.set_join_u(true);
        base.set_join_v(true);
        base.set_twist_u(true);
        base.set_twist_v(true);
        base.set_clockwise_ordering(false);
        base.set_derivatives_available(true);

        Self {
            base,
            z_scale: 0.125,
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

    /// VTK: `vtkParametricBoy::SetZScale`.
    pub fn set_z_scale(&mut self, value: f64) {
        if self.z_scale != value {
            self.z_scale = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricBoy::GetZScale`.
    pub fn get_z_scale(&self) -> f64 {
        self.z_scale
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

impl ParametricFunctionApi for ParametricBoy {
    fn parametric_function(&self) -> &ParametricFunction {
        &self.base
    }

    fn parametric_function_mut(&mut self) -> &mut ParametricFunction {
        &mut self.base
    }

    /// VTK: `vtkParametricBoy::GetDimension`.
    fn get_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkParametricBoy::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], duvw: &mut [f64; 9]) {
        let u = uvw[0];
        let v = uvw[1];

        duvw.fill(0.0);

        let cu = u.cos();
        let su = u.sin();
        let sv = v.sin();

        let x = cu * sv;
        let y = su * sv;
        let z = v.cos();

        let x2 = x * x;
        let x3 = x2 * x;
        let x4 = x3 * x;
        let y2 = y * y;
        let y3 = y2 * y;
        let y4 = y3 * y;
        let z2 = z * z;
        let z3 = z2 * z;
        let z4 = z3 * z;

        let sr3 = 3.0_f64.sqrt();

        pt[0] = 0.5
            * (2.0 * x2 - y2 - z2
                + 2.0 * y * z * (y2 - z2)
                + z * x * (x2 - z2)
                + x * y * (y2 - x2));
        pt[1] = sr3 / 2.0 * (y2 - z2 + (z * x * (z2 - x2) + x * y * (y2 - x2)));
        pt[2] = self.z_scale
            * (x + y + z)
            * ((x + y + z) * (x + y + z) * (x + y + z) + 4.0 * (y - x) * (z - y) * (x - z));

        duvw[0] = -0.5 * x4 - z3 * x + 3.0 * y2 * x2 - 1.5 * z * x2 * y + 3.0 * z * x * y2
            - 3.0 * y * x
            - 0.5 * y4
            + 0.5 * z3 * y;
        duvw[3] = (1.5 * z2 * x2 + 2.0 * z * x - 0.5 * z4) * cu
            + (-2.0 * z * x3 + 2.0 * z * x * y2 + 3.0 * z2 * y2 - z * y - z4) * su
            + (-0.5 * x3 + 1.5 * z2 * x - y3 + 3.0 * z2 * y + z) * sv;
        duvw[1] = -0.5 * sr3 * x4 + 3.0 * sr3 * y2 * x2 + 1.5 * sr3 * z * x2 * y + sr3 * y * x
            - 0.5 * sr3 * y4
            - 0.5 * sr3 * z3 * y;
        duvw[4] = (-1.5 * sr3 * z2 * x2 + 0.5 * sr3 * z4) * cu
            + (-2.0 * sr3 * z * x3 + 2.0 * sr3 * z * y2 * x + sr3 * z * y) * su
            + (0.5 * sr3 * x3 - 1.5 * sr3 * z2 * x + sr3 * z) * sv;

        duvw[2] = x4 + z * x3 + z2 * x2 + x3 * y - 3.0 * x2 * y2 + 3.0 * z * x2 * y
            - y3 * x
            - z * y3
            - z2 * y2
            - z3 * y;
        duvw[5] = (z3 * x + z4) * cu
            + (4.0 * z * x3
                + 3.0 * z * x2 * y
                + 4.0 * z2 * x2
                + 4.0 * z2 * x * y
                + 3.0 * z3 * x
                + 3.0 * z2 * y2
                + z3 * y)
                * su
            + (-x2 * y - z * x2 - x * y2 - 3.0 * z * x * y - 3.0 * z2 * x - y3 - z * y2) * sv;
    }

    /// VTK: `vtkParametricBoy::EvaluateScalar`.
    fn evaluate_scalar(&self, _uvw: [f64; 3], _pt: [f64; 3], _duvw: [f64; 9]) -> f64 {
        0.0
    }
}

impl Default for ParametricBoy {
    fn default() -> Self {
        Self::new()
    }
}
