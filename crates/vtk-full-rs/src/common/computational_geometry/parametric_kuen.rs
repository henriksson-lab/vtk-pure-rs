use crate::common::{
    computational_geometry::{ParametricFunction, ParametricFunctionApi},
    core::{math::pi, VtkMTimeType},
};

/// VTK: `vtkParametricKuen`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricKuen {
    base: ParametricFunction,
    delta_v0: f64,
}

impl ParametricKuen {
    /// VTK: `vtkParametricKuen::New`.
    pub fn new() -> Self {
        let mut base = ParametricFunction::with_class_name("vtkParametricKuen");
        base.set_minimum_u(-4.5);
        base.set_maximum_u(4.5);
        base.set_minimum_v(0.0);
        base.set_maximum_v(pi());
        base.set_join_u(false);
        base.set_join_v(false);
        base.set_twist_u(false);
        base.set_twist_v(false);
        base.set_clockwise_ordering(false);
        base.set_derivatives_available(true);

        Self {
            base,
            delta_v0: 0.05,
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

    /// VTK: `vtkParametricKuen::SetDeltaV0`.
    pub fn set_delta_v0(&mut self, value: f64) {
        if self.delta_v0 != value {
            self.delta_v0 = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricKuen::GetDeltaV0`.
    pub fn get_delta_v0(&self) -> f64 {
        self.delta_v0
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

impl ParametricFunctionApi for ParametricKuen {
    fn parametric_function(&self) -> &ParametricFunction {
        &self.base
    }

    fn parametric_function_mut(&mut self) -> &mut ParametricFunction {
        &mut self.base
    }

    /// VTK: `vtkParametricKuen::GetDimension`.
    fn get_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkParametricKuen::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], duvw: &mut [f64; 9]) {
        let u = uvw[0];
        let mut v = uvw[1];

        if v == 0.0 {
            v = self.delta_v0;
        }

        duvw.fill(0.0);

        let cosu = u.cos();
        let cosv = v.cos();
        let sinu = u.sin();
        let sinv = v.sin();
        let sinv2 = sinv * sinv;
        let u2 = u * u;
        let denom_1 = 1.0 + u2 * sinv2;
        let denom_2 = if sinv.abs() < 1.0e-9 {
            0.0
        } else {
            u2 + 1.0 / sinv2
        };

        let log_tan_v_over_2 = if pi() - f32::EPSILON as f64 <= v && v <= pi() + f32::EPSILON as f64
        {
            1.0
        } else {
            (v / 2.0).tan().ln()
        };

        pt[0] = 2.0 * sinv * (sinu - u * cosu) / denom_1;
        pt[1] = 2.0 * sinv * (cosu + u * sinu) / denom_1;
        pt[2] = log_tan_v_over_2 + 2.0 * cosv / denom_1;

        duvw[1] = (2.0 * u * sinv * (cosu + ((u2 - 2.0) * cosu - 2.0 * u * sinu) * sinv2))
            / (denom_1 * denom_1);

        if denom_2 == 0.0 || sinv == 0.0 {
            duvw[0] = 0.0;
            duvw[2] = 0.0;
        } else {
            duvw[0] = (2.0 * u / sinv * (2.0 * u * cosu + (u2 - 2.0 + 1.0 / sinv2) * sinu))
                / (denom_2 * denom_2);
            duvw[2] = -4.0 * u * cosv / (denom_2 * denom_2 * sinv2);
        }

        duvw[3] = 2.0 * cosv * (u2 * sinv2 - 1.0) * (u * cosu - sinu) / (denom_1 * denom_1);
        duvw[4] = 2.0 * cosv * (1.0 - u2 * sinv2) * (cosu + u * sinu) / (denom_1 * denom_1);
        duvw[5] = 1.0 / sinv - (2.0 + u2 * (3.0 + (2.0 * u).cos())) * sinv / (denom_1 * denom_1);
    }

    /// VTK: `vtkParametricKuen::EvaluateScalar`.
    fn evaluate_scalar(&self, _uvw: [f64; 3], _pt: [f64; 3], _duvw: [f64; 9]) -> f64 {
        0.0
    }
}

impl Default for ParametricKuen {
    fn default() -> Self {
        Self::new()
    }
}
