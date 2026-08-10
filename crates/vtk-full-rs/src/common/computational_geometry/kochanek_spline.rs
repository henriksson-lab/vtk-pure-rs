use crate::common::{
    core::VtkMTimeType,
    data_model::{Spline, SplineApi},
};

/// VTK: `vtkKochanekSpline`.
#[derive(Debug, Clone, PartialEq)]
pub struct KochanekSpline {
    base: Spline,
    default_bias: f64,
    default_tension: f64,
    default_continuity: f64,
}

impl KochanekSpline {
    /// VTK: `vtkKochanekSpline::New`.
    pub fn new() -> Self {
        Self {
            base: Spline::with_class_name("vtkKochanekSpline"),
            default_bias: 0.0,
            default_tension: 0.0,
            default_continuity: 0.0,
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

    /// VTK: `vtkSpline::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.base.get_m_time()
    }

    /// VTK: `vtkSpline::SetParametricRange`.
    pub fn set_parametric_range(&mut self, t_min: f64, t_max: f64) {
        self.base.set_parametric_range(t_min, t_max);
    }

    /// VTK: `vtkSpline::SetParametricRange`.
    pub fn set_parametric_range_from_slice(&mut self, t_range: [f64; 2]) {
        self.base.set_parametric_range_from_slice(t_range);
    }

    /// VTK: `vtkSpline::GetParametricRange`.
    pub fn get_parametric_range(&self) -> [f64; 2] {
        self.base.get_parametric_range()
    }

    /// VTK: `vtkSpline::SetClampValue`.
    pub fn set_clamp_value(&mut self, value: bool) {
        self.base.set_clamp_value(value);
    }

    /// VTK: `vtkSpline::GetClampValue`.
    pub fn get_clamp_value(&self) -> bool {
        self.base.get_clamp_value()
    }

    /// VTK: `vtkSpline::ClampValueOn`.
    pub fn clamp_value_on(&mut self) {
        self.base.clamp_value_on();
    }

    /// VTK: `vtkSpline::ClampValueOff`.
    pub fn clamp_value_off(&mut self) {
        self.base.clamp_value_off();
    }

    /// VTK: `vtkSpline::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> i32 {
        self.base.get_number_of_points()
    }

    /// VTK: `vtkSpline::AddPoint`.
    pub fn add_point(&mut self, t: f64, x: f64) {
        self.base.add_point(t, x);
    }

    /// VTK: `vtkSpline::FillFromDataPointer`.
    pub fn fill_from_data_pointer(&mut self, nb: i32, data: &[f64]) {
        self.base.fill_from_data_pointer(nb, data);
    }

    /// VTK: `vtkSpline::RemovePoint`.
    pub fn remove_point(&mut self, t: f64) {
        self.base.remove_point(t);
    }

    /// VTK: `vtkSpline::RemoveAllPoints`.
    pub fn remove_all_points(&mut self) {
        self.base.remove_all_points();
    }

    /// VTK: `vtkSpline::SetClosed`.
    pub fn set_closed(&mut self, value: bool) {
        self.base.set_closed(value);
    }

    /// VTK: `vtkSpline::GetClosed`.
    pub fn get_closed(&self) -> bool {
        self.base.get_closed()
    }

    /// VTK: `vtkSpline::ClosedOn`.
    pub fn closed_on(&mut self) {
        self.base.closed_on();
    }

    /// VTK: `vtkSpline::ClosedOff`.
    pub fn closed_off(&mut self) {
        self.base.closed_off();
    }

    /// VTK: `vtkSpline::SetLeftConstraint`.
    pub fn set_left_constraint(&mut self, value: i32) {
        self.base.set_left_constraint(value);
    }

    /// VTK: `vtkSpline::GetLeftConstraint`.
    pub fn get_left_constraint(&self) -> i32 {
        self.base.get_left_constraint()
    }

    /// VTK: `vtkSpline::SetRightConstraint`.
    pub fn set_right_constraint(&mut self, value: i32) {
        self.base.set_right_constraint(value);
    }

    /// VTK: `vtkSpline::GetRightConstraint`.
    pub fn get_right_constraint(&self) -> i32 {
        self.base.get_right_constraint()
    }

    /// VTK: `vtkSpline::SetLeftValue`.
    pub fn set_left_value(&mut self, value: f64) {
        self.base.set_left_value(value);
    }

    /// VTK: `vtkSpline::GetLeftValue`.
    pub fn get_left_value(&self) -> f64 {
        self.base.get_left_value()
    }

    /// VTK: `vtkSpline::SetRightValue`.
    pub fn set_right_value(&mut self, value: f64) {
        self.base.set_right_value(value);
    }

    /// VTK: `vtkSpline::GetRightValue`.
    pub fn get_right_value(&self) -> f64 {
        self.base.get_right_value()
    }

    /// VTK: `vtkKochanekSpline::SetDefaultBias`.
    pub fn set_default_bias(&mut self, value: f64) {
        if self.default_bias != value {
            self.default_bias = value;
            self.modified();
        }
    }

    /// VTK: `vtkKochanekSpline::GetDefaultBias`.
    pub fn get_default_bias(&self) -> f64 {
        self.default_bias
    }

    /// VTK: `vtkKochanekSpline::SetDefaultTension`.
    pub fn set_default_tension(&mut self, value: f64) {
        if self.default_tension != value {
            self.default_tension = value;
            self.modified();
        }
    }

    /// VTK: `vtkKochanekSpline::GetDefaultTension`.
    pub fn get_default_tension(&self) -> f64 {
        self.default_tension
    }

    /// VTK: `vtkKochanekSpline::SetDefaultContinuity`.
    pub fn set_default_continuity(&mut self, value: f64) {
        if self.default_continuity != value {
            self.default_continuity = value;
            self.modified();
        }
    }

    /// VTK: `vtkKochanekSpline::GetDefaultContinuity`.
    pub fn get_default_continuity(&self) -> f64 {
        self.default_continuity
    }

    /// VTK: `vtkKochanekSpline::Evaluate`.
    pub fn evaluate(&mut self, mut t: f64) -> f64 {
        if self.base.get_compute_time() < self.base.get_m_time() {
            self.compute();
        }

        let mut size = self.base.points().len();
        if size < 2 {
            return 0.0;
        }

        if self.base.get_closed() {
            size += 1;
        }

        let intervals = self.base.intervals();
        let coefficients = self.base.coefficients();
        t = t.max(intervals[0]);
        t = t.min(intervals[size - 1]);

        let index = self.base.find_index(size, t);
        let t = (t - intervals[index]) / (intervals[index + 1] - intervals[index]);

        t * (t * (t * coefficients[index][3] + coefficients[index][2]) + coefficients[index][1])
            + coefficients[index][0]
    }

    /// VTK: `vtkKochanekSpline::Compute`.
    pub fn compute(&mut self) {
        let mut size = self.base.points().len();
        if size < 2 {
            return;
        }

        let mut intervals = Vec::with_capacity(if self.base.get_closed() {
            size + 1
        } else {
            size
        });
        intervals.extend(self.base.points().iter().map(|point| point[0]));

        let mut dependent = Vec::with_capacity(intervals.capacity());
        dependent.extend(self.base.points().iter().map(|point| point[1]));

        if self.base.get_closed() {
            size += 1;
            let explicit_range = self.base.get_parametric_range_storage();
            if explicit_range[0] != explicit_range[1] {
                intervals.push(explicit_range[1]);
            } else {
                intervals.push(intervals[size - 2] + 1.0);
            }
            dependent.push(dependent[0]);
        }

        let mut coefficients = vec![[0.0; 4]; size];
        self.fit_1d(
            size,
            &intervals,
            &dependent,
            self.default_tension,
            self.default_bias,
            self.default_continuity,
            &mut coefficients,
            self.base.get_left_constraint(),
            self.base.get_left_value(),
            self.base.get_right_constraint(),
            self.base.get_right_value(),
        );

        self.base.set_intervals(intervals);
        self.base.set_coefficients(coefficients);
        self.base.set_compute_time_to_m_time();
    }

    /// VTK: `vtkKochanekSpline::DeepCopy`.
    pub fn deep_copy(&mut self, other: &dyn SplineApi) {
        if let Some(spline) = other.as_any().downcast_ref::<Self>() {
            self.default_bias = spline.default_bias;
            self.default_tension = spline.default_tension;
            self.default_continuity = spline.default_continuity;
        }

        self.base.deep_copy(other.spline());
    }

    /// VTK: `vtkKochanekSpline::Fit1D`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fit_1d(
        &self,
        size: usize,
        x: &[f64],
        y: &[f64],
        tension: f64,
        bias: f64,
        continuity: f64,
        coefficients: &mut [[f64; 4]],
        left_constraint: i32,
        left_value: f64,
        right_constraint: i32,
        right_value: f64,
    ) {
        let n = size - 1;

        for i in 1..n {
            let cs = y[i] - y[i - 1];
            let cd = y[i + 1] - y[i];

            let mut ds = cs * ((1.0 - tension) * (1.0 - continuity) * (1.0 + bias)) / 2.0
                + cd * ((1.0 - tension) * (1.0 + continuity) * (1.0 - bias)) / 2.0;
            let mut dd = cs * ((1.0 - tension) * (1.0 + continuity) * (1.0 + bias)) / 2.0
                + cd * ((1.0 - tension) * (1.0 - continuity) * (1.0 - bias)) / 2.0;

            let n1 = x[i + 1] - x[i];
            let n0 = x[i] - x[i - 1];
            ds *= 2.0 * n0 / (n0 + n1);
            dd *= 2.0 * n1 / (n0 + n1);

            coefficients[i][0] = y[i];
            coefficients[i][1] = dd;
            coefficients[i][2] = ds;
        }

        coefficients[0][0] = y[0];
        coefficients[n][0] = y[n];
        coefficients[n][1] = 0.0;
        coefficients[n][2] = 0.0;
        coefficients[n][3] = 0.0;

        if self.base.get_closed() {
            let cs = y[n] - y[n - 1];
            let cd = y[1] - y[0];

            let mut ds = cs * ((1.0 - tension) * (1.0 - continuity) * (1.0 + bias)) / 2.0
                + cd * ((1.0 - tension) * (1.0 + continuity) * (1.0 - bias)) / 2.0;
            let mut dd = cs * ((1.0 - tension) * (1.0 + continuity) * (1.0 + bias)) / 2.0
                + cd * ((1.0 - tension) * (1.0 - continuity) * (1.0 - bias)) / 2.0;

            let n1 = x[1] - x[0];
            let n0 = x[n] - x[n - 1];
            ds *= 2.0 * n0 / (n0 + n1);
            dd *= 2.0 * n1 / (n0 + n1);

            coefficients[0][1] = dd;
            coefficients[0][2] = ds;
            coefficients[n][1] = dd;
            coefficients[n][2] = ds;
        } else {
            const VTK_EPSILON: f64 = 0.0001;

            match left_constraint {
                0 => coefficients[0][1] = self.base.compute_left_derivative(),
                1 => coefficients[0][1] = left_value,
                2 => {
                    coefficients[0][1] =
                        (6.0 * (y[1] - y[0]) - 2.0 * coefficients[1][2] - left_value) / 4.0;
                }
                3 => {
                    if (left_value > (-2.0 + VTK_EPSILON)) || (left_value < (-2.0 - VTK_EPSILON)) {
                        coefficients[0][1] = (3.0 * (1.0 + left_value) * (y[1] - y[0])
                            - (1.0 + 2.0 * left_value) * coefficients[1][2])
                            / (2.0 + left_value);
                    } else {
                        coefficients[0][1] = 0.0;
                    }
                }
                _ => unreachable!("vtkSpline clamps LeftConstraint into [0, 3]"),
            }

            match right_constraint {
                0 => coefficients[n][2] = self.base.compute_right_derivative(),
                1 => coefficients[n][2] = right_value,
                2 => {
                    coefficients[n][2] = (6.0 * (y[n] - y[n - 1]) - 2.0 * coefficients[n - 1][1]
                        + right_value)
                        / 4.0;
                }
                3 => {
                    if (right_value > (-2.0 + VTK_EPSILON)) || (right_value < (-2.0 - VTK_EPSILON))
                    {
                        coefficients[n][2] = (3.0 * (1.0 + right_value) * (y[n] - y[n - 1])
                            - (1.0 + 2.0 * right_value) * coefficients[n - 1][1])
                            / (2.0 + right_value);
                    } else {
                        coefficients[n][2] = 0.0;
                    }
                }
                _ => unreachable!("vtkSpline clamps RightConstraint into [0, 3]"),
            }
        }

        for i in 0..n {
            coefficients[i][2] =
                -3.0 * y[i] + 3.0 * y[i + 1] - 2.0 * coefficients[i][1] - coefficients[i + 1][2];
            coefficients[i][3] =
                2.0 * y[i] - 2.0 * y[i + 1] + coefficients[i][1] + coefficients[i + 1][2];
        }
    }
}

impl Default for KochanekSpline {
    fn default() -> Self {
        Self::new()
    }
}

impl SplineApi for KochanekSpline {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn spline(&self) -> &Spline {
        &self.base
    }

    fn spline_mut(&mut self) -> &mut Spline {
        &mut self.base
    }

    fn compute(&mut self) {
        KochanekSpline::compute(self);
    }

    fn evaluate(&mut self, t: f64) -> f64 {
        KochanekSpline::evaluate(self, t)
    }

    fn clone_box(&self) -> Box<dyn SplineApi> {
        Box::new(self.clone())
    }
}
