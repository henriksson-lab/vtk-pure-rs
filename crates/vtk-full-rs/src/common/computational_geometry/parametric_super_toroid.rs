use crate::common::{
    computational_geometry::{ParametricFunction, ParametricFunctionApi},
    core::{math::pi, VtkMTimeType},
};

/// VTK: `vtkParametricSuperToroid`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricSuperToroid {
    base: ParametricFunction,
    ring_radius: f64,
    cross_section_radius: f64,
    x_radius: f64,
    y_radius: f64,
    z_radius: f64,
    n1: f64,
    n2: f64,
}

fn sgn_power(x: f64, n: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if n == 0.0 {
        return 1.0;
    }
    let sgn = if x < 0.0 { -1.0 } else { 1.0 };
    sgn * x.abs().powf(n)
}

impl ParametricSuperToroid {
    /// VTK: `vtkParametricSuperToroid::New`.
    pub fn new() -> Self {
        let mut base = ParametricFunction::with_class_name("vtkParametricSuperToroid");
        base.set_minimum_u(0.0);
        base.set_maximum_u(2.0 * pi());
        base.set_minimum_v(0.0);
        base.set_maximum_v(2.0 * pi());
        base.set_join_u(false);
        base.set_join_v(false);
        base.set_twist_u(false);
        base.set_twist_v(false);
        base.set_clockwise_ordering(false);
        base.set_derivatives_available(false);

        Self {
            base,
            ring_radius: 1.0,
            cross_section_radius: 0.5,
            x_radius: 1.0,
            y_radius: 1.0,
            z_radius: 1.0,
            n1: 1.0,
            n2: 1.0,
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

    /// VTK: `vtkParametricSuperToroid::SetRingRadius`.
    pub fn set_ring_radius(&mut self, value: f64) {
        if self.ring_radius != value {
            self.ring_radius = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetRingRadius`.
    pub fn get_ring_radius(&self) -> f64 {
        self.ring_radius
    }

    /// VTK: `vtkParametricSuperToroid::SetCrossSectionRadius`.
    pub fn set_cross_section_radius(&mut self, value: f64) {
        if self.cross_section_radius != value {
            self.cross_section_radius = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetCrossSectionRadius`.
    pub fn get_cross_section_radius(&self) -> f64 {
        self.cross_section_radius
    }

    /// VTK: `vtkParametricSuperToroid::SetXRadius`.
    pub fn set_x_radius(&mut self, value: f64) {
        if self.x_radius != value {
            self.x_radius = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetXRadius`.
    pub fn get_x_radius(&self) -> f64 {
        self.x_radius
    }

    /// VTK: `vtkParametricSuperToroid::SetYRadius`.
    pub fn set_y_radius(&mut self, value: f64) {
        if self.y_radius != value {
            self.y_radius = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetYRadius`.
    pub fn get_y_radius(&self) -> f64 {
        self.y_radius
    }

    /// VTK: `vtkParametricSuperToroid::SetZRadius`.
    pub fn set_z_radius(&mut self, value: f64) {
        if self.z_radius != value {
            self.z_radius = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetZRadius`.
    pub fn get_z_radius(&self) -> f64 {
        self.z_radius
    }

    /// VTK: `vtkParametricSuperToroid::SetN1`.
    pub fn set_n1(&mut self, value: f64) {
        if self.n1 != value {
            self.n1 = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetN1`.
    pub fn get_n1(&self) -> f64 {
        self.n1
    }

    /// VTK: `vtkParametricSuperToroid::SetN2`.
    pub fn set_n2(&mut self, value: f64) {
        if self.n2 != value {
            self.n2 = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSuperToroid::GetN2`.
    pub fn get_n2(&self) -> f64 {
        self.n2
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

impl ParametricFunctionApi for ParametricSuperToroid {
    fn parametric_function(&self) -> &ParametricFunction {
        &self.base
    }

    fn parametric_function_mut(&mut self) -> &mut ParametricFunction {
        &mut self.base
    }

    /// VTK: `vtkParametricSuperToroid::GetDimension`.
    fn get_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkParametricSuperToroid::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], duvw: &mut [f64; 9]) {
        let u = uvw[0];
        let v = uvw[1];

        pt.fill(0.0);
        duvw.fill(0.0);

        let cu = u.cos();
        let su = u.sin();
        let cv = v.cos();
        let sv = v.sin();

        let tmp = self.ring_radius + self.cross_section_radius * sgn_power(cv, self.n2);

        pt[0] = self.x_radius * tmp * sgn_power(su, self.n1);
        pt[1] = self.y_radius * tmp * sgn_power(cu, self.n1);
        pt[2] = self.z_radius * self.cross_section_radius * sgn_power(sv, self.n2);
    }

    /// VTK: `vtkParametricSuperToroid::EvaluateScalar`.
    fn evaluate_scalar(&self, _uvw: [f64; 3], _pt: [f64; 3], _duvw: [f64; 9]) -> f64 {
        0.0
    }
}

impl Default for ParametricSuperToroid {
    fn default() -> Self {
        Self::new()
    }
}
