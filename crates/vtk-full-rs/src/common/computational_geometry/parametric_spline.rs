use std::cell::{Cell, Ref, RefCell};

use crate::common::{
    computational_geometry::{CardinalSpline, ParametricFunction, ParametricFunctionApi},
    core::{Points, VtkIdType, VtkMTimeType, VTK_DOUBLE},
    data_model::SplineApi,
};

/// VTK: `vtkParametricSpline`.
#[derive(Debug, Clone)]
pub struct ParametricSpline {
    base: ParametricFunction,
    points: Option<Points>,
    x_spline: RefCell<Option<Box<dyn SplineApi>>>,
    y_spline: RefCell<Option<Box<dyn SplineApi>>>,
    z_spline: RefCell<Option<Box<dyn SplineApi>>>,
    closed: bool,
    left_constraint: i32,
    right_constraint: i32,
    left_value: f64,
    right_value: f64,
    parameterize_by_length: bool,
    initialize_time: Cell<VtkMTimeType>,
    length: Cell<f64>,
    closed_length: Cell<f64>,
}

impl ParametricSpline {
    /// VTK: `vtkParametricSpline::New`.
    pub fn new() -> Self {
        let mut base = ParametricFunction::with_class_name("vtkParametricSpline");
        base.set_minimum_u(0.0);
        base.set_maximum_u(1.0);
        base.set_join_u(false);
        base.set_twist_u(false);
        base.set_derivatives_available(false);

        Self {
            base,
            points: None,
            x_spline: RefCell::new(Some(Box::new(CardinalSpline::new()))),
            y_spline: RefCell::new(Some(Box::new(CardinalSpline::new()))),
            z_spline: RefCell::new(Some(Box::new(CardinalSpline::new()))),
            closed: false,
            left_constraint: 1,
            right_constraint: 1,
            left_value: 0.0,
            right_value: 0.0,
            parameterize_by_length: true,
            initialize_time: Cell::new(0),
            length: Cell::new(0.0),
            closed_length: Cell::new(0.0),
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

    /// VTK: `vtkParametricSpline::SetXSpline`.
    pub fn set_x_spline(&mut self, spline: Option<Box<dyn SplineApi>>) {
        *self.x_spline.borrow_mut() = spline;
        self.modified();
    }

    /// VTK: `vtkParametricSpline::SetYSpline`.
    pub fn set_y_spline(&mut self, spline: Option<Box<dyn SplineApi>>) {
        *self.y_spline.borrow_mut() = spline;
        self.modified();
    }

    /// VTK: `vtkParametricSpline::SetZSpline`.
    pub fn set_z_spline(&mut self, spline: Option<Box<dyn SplineApi>>) {
        *self.z_spline.borrow_mut() = spline;
        self.modified();
    }

    /// VTK: `vtkParametricSpline::GetXSpline`.
    pub fn get_x_spline(&self) -> Ref<'_, Option<Box<dyn SplineApi>>> {
        self.x_spline.borrow()
    }

    /// VTK: `vtkParametricSpline::GetYSpline`.
    pub fn get_y_spline(&self) -> Ref<'_, Option<Box<dyn SplineApi>>> {
        self.y_spline.borrow()
    }

    /// VTK: `vtkParametricSpline::GetZSpline`.
    pub fn get_z_spline(&self) -> Ref<'_, Option<Box<dyn SplineApi>>> {
        self.z_spline.borrow()
    }

    /// VTK: `vtkParametricSpline::SetPoints`.
    pub fn set_points(&mut self, points: Option<Points>) {
        self.points = points;
        self.modified();
    }

    /// VTK: `vtkParametricSpline::GetPoints`.
    pub fn get_points(&self) -> Option<&Points> {
        self.points.as_ref()
    }

    /// VTK: `vtkParametricSpline::SetNumberOfPoints`.
    pub fn set_number_of_points(&mut self, num_pts: VtkIdType) {
        if self.points.is_none() {
            self.set_points(Some(Points::new_with_data_type(VTK_DOUBLE)));
        }
        if let Some(points) = self.points.as_mut() {
            points.set_number_of_points(num_pts);
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::SetPoint`.
    pub fn set_point(&mut self, index: VtkIdType, x: f64, y: f64, z: f64) {
        if let Some(points) = self.points.as_mut() {
            points.set_point(index, [x, y, z]);
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::SetClosed`.
    pub fn set_closed(&mut self, value: bool) {
        if self.closed != value {
            self.closed = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::GetClosed`.
    pub fn get_closed(&self) -> bool {
        self.closed
    }

    /// VTK: `vtkParametricSpline::ClosedOn`.
    pub fn closed_on(&mut self) {
        self.set_closed(true);
    }

    /// VTK: `vtkParametricSpline::ClosedOff`.
    pub fn closed_off(&mut self) {
        self.set_closed(false);
    }

    /// VTK: `vtkParametricSpline::SetParameterizeByLength`.
    pub fn set_parameterize_by_length(&mut self, value: bool) {
        if self.parameterize_by_length != value {
            self.parameterize_by_length = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::GetParameterizeByLength`.
    pub fn get_parameterize_by_length(&self) -> bool {
        self.parameterize_by_length
    }

    /// VTK: `vtkParametricSpline::ParameterizeByLengthOn`.
    pub fn parameterize_by_length_on(&mut self) {
        self.set_parameterize_by_length(true);
    }

    /// VTK: `vtkParametricSpline::ParameterizeByLengthOff`.
    pub fn parameterize_by_length_off(&mut self) {
        self.set_parameterize_by_length(false);
    }

    /// VTK: `vtkParametricSpline::SetLeftConstraint`.
    pub fn set_left_constraint(&mut self, value: i32) {
        let value = value.clamp(0, 3);
        if self.left_constraint != value {
            self.left_constraint = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::GetLeftConstraint`.
    pub fn get_left_constraint(&self) -> i32 {
        self.left_constraint
    }

    /// VTK: `vtkParametricSpline::SetRightConstraint`.
    pub fn set_right_constraint(&mut self, value: i32) {
        let value = value.clamp(0, 3);
        if self.right_constraint != value {
            self.right_constraint = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::GetRightConstraint`.
    pub fn get_right_constraint(&self) -> i32 {
        self.right_constraint
    }

    /// VTK: `vtkParametricSpline::SetLeftValue`.
    pub fn set_left_value(&mut self, value: f64) {
        if self.left_value != value {
            self.left_value = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::GetLeftValue`.
    pub fn get_left_value(&self) -> f64 {
        self.left_value
    }

    /// VTK: `vtkParametricSpline::SetRightValue`.
    pub fn set_right_value(&mut self, value: f64) {
        if self.right_value != value {
            self.right_value = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricSpline::GetRightValue`.
    pub fn get_right_value(&self) -> f64 {
        self.right_value
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

    /// VTK: `vtkParametricSpline::Initialize`.
    fn initialize(&self) -> bool {
        let points = match self.points.as_ref() {
            Some(points) => points,
            None => return false,
        };

        if self.x_spline.borrow().is_none()
            || self.y_spline.borrow().is_none()
            || self.z_spline.borrow().is_none()
        {
            return false;
        }

        {
            let mut x_spline = self.x_spline.borrow_mut();
            self.configure_spline(x_spline.as_deref_mut().expect("checked"));
        }
        {
            let mut y_spline = self.y_spline.borrow_mut();
            self.configure_spline(y_spline.as_deref_mut().expect("checked"));
        }
        {
            let mut z_spline = self.z_spline.borrow_mut();
            self.configure_spline(z_spline.as_deref_mut().expect("checked"));
        }

        let npts = points.get_number_of_points();
        if npts < 1 {
            return false;
        }
        if npts < 2 {
            self.length.set(0.0);
            self.closed_length.set(0.0);
            return true;
        }

        if self.parameterize_by_length {
            let mut x_prev = points.get_point(0);
            let mut length = 0.0;
            for i in 1..npts {
                let x = points.get_point(i);
                length += distance_between_points(x, x_prev);
                x_prev = x;
            }
            self.length.set(length);

            if length <= 0.0 {
                self.closed_length.set(0.0);
                return true;
            }
            if self.closed {
                let x = points.get_point(0);
                self.closed_length
                    .set(length + distance_between_points(x, x_prev));
            }
        } else {
            self.length.set((npts - 1) as f64);
            if self.closed {
                self.closed_length.set(npts as f64);
            }
        }

        self.x_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .remove_all_points();
        self.y_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .remove_all_points();
        self.z_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .remove_all_points();

        let range_max = if self.closed {
            self.closed_length.get()
        } else {
            self.length.get()
        };
        self.x_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .set_parametric_range(0.0, range_max);
        self.y_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .set_parametric_range(0.0, range_max);
        self.z_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .set_parametric_range(0.0, range_max);

        if self.parameterize_by_length {
            let mut x_prev = points.get_point(0);
            let mut len = 0.0;
            for i in 0..npts {
                let x = points.get_point(i);
                len += distance_between_points(x, x_prev);
                self.add_spline_points(len, x);
                x_prev = x;
            }
        } else {
            for i in 0..npts {
                self.add_spline_points(i as f64, points.get_point(i));
            }
        }

        self.initialize_time.set(self.get_m_time());
        true
    }

    fn configure_spline(&self, spline: &mut dyn SplineApi) {
        spline.set_closed(self.get_closed());
        spline.set_left_constraint(self.get_left_constraint());
        spline.set_right_constraint(self.get_right_constraint());
        spline.set_left_value(self.get_left_value());
        spline.set_right_value(self.get_right_value());
    }

    fn add_spline_points(&self, t: f64, point: [f64; 3]) {
        self.x_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .add_point(t, point[0]);
        self.y_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .add_point(t, point[1]);
        self.z_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("checked")
            .add_point(t, point[2]);
    }
}

impl ParametricFunctionApi for ParametricSpline {
    fn parametric_function(&self) -> &ParametricFunction {
        &self.base
    }

    fn parametric_function_mut(&mut self) -> &mut ParametricFunction {
        &mut self.base
    }

    /// VTK: `vtkParametricSpline::GetDimension`.
    fn get_dimension(&self) -> i32 {
        1
    }

    /// VTK: `vtkParametricSpline::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], _duvw: &mut [f64; 9]) {
        if self.initialize_time.get() < self.get_m_time() && !self.initialize() {
            return;
        }

        let mut t = uvw[0].clamp(0.0, 1.0);
        if self.closed {
            t *= self.closed_length.get();
        } else {
            t *= self.length.get();
        }

        if self.length.get() == 0.0 {
            if let Some(points) = self.points.as_ref() {
                if points.get_number_of_points() > 0 {
                    *pt = points.get_point(0);
                    return;
                }
            }
        }

        pt[0] = self
            .x_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("Initialize checked spline presence")
            .evaluate(t);
        pt[1] = self
            .y_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("Initialize checked spline presence")
            .evaluate(t);
        pt[2] = self
            .z_spline
            .borrow_mut()
            .as_deref_mut()
            .expect("Initialize checked spline presence")
            .evaluate(t);
    }

    /// VTK: `vtkParametricSpline::EvaluateScalar`.
    fn evaluate_scalar(&self, uvw: [f64; 3], _pt: [f64; 3], _duvw: [f64; 9]) -> f64 {
        if self.initialize_time.get() < self.get_m_time() && !self.initialize() {
            return 0.0;
        }

        uvw[0]
    }
}

impl Default for ParametricSpline {
    fn default() -> Self {
        Self::new()
    }
}

fn distance_between_points(left: [f64; 3], right: [f64; 3]) -> f64 {
    let dx = left[0] - right[0];
    let dy = left[1] - right[1];
    let dz = left[2] - right[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}
