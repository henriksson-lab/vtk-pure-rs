use crate::common::{
    core::VtkMTimeType,
    data_model::{Spline, SplineApi},
};

/// VTK: `vtkCardinalSpline`.
#[derive(Debug, Clone, PartialEq)]
pub struct CardinalSpline {
    base: Spline,
}

impl CardinalSpline {
    /// VTK: `vtkCardinalSpline::New`.
    pub fn new() -> Self {
        Self {
            base: Spline::with_class_name("vtkCardinalSpline"),
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

    /// VTK: `vtkCardinalSpline::Evaluate`.
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
        let t = t - intervals[index];

        t * (t * (t * coefficients[index][3] + coefficients[index][2]) + coefficients[index][1])
            + coefficients[index][0]
    }

    /// VTK: `vtkCardinalSpline::Compute`.
    pub fn compute(&mut self) {
        let mut size = self.base.points().len();
        if size < 2 {
            return;
        }

        if !self.base.get_closed() {
            let intervals: Vec<f64> = self.base.points().iter().map(|point| point[0]).collect();
            let dependent: Vec<f64> = self.base.points().iter().map(|point| point[1]).collect();
            let mut work = vec![0.0; size];
            let mut coefficients = vec![[0.0; 4]; size];

            self.fit_1d(
                size,
                &intervals,
                &dependent,
                &mut work,
                &mut coefficients,
                self.base.get_left_constraint(),
                self.base.get_left_value(),
                self.base.get_right_constraint(),
                self.base.get_right_value(),
            );

            self.base.set_intervals(intervals);
            self.base.set_coefficients(coefficients);
        } else {
            size += 1;
            let mut intervals = Vec::with_capacity(size);
            intervals.extend(self.base.points().iter().map(|point| point[0]));
            let explicit_range = self.base.get_parametric_range_storage();
            if explicit_range[0] != explicit_range[1] {
                intervals.push(explicit_range[1]);
            } else {
                intervals.push(intervals[size - 2] + 1.0);
            }

            let mut dependent = Vec::with_capacity(size);
            dependent.extend(self.base.points().iter().map(|point| point[1]));
            dependent.push(dependent[0]);

            let mut work = vec![0.0; size];
            let mut coefficients = vec![[0.0; 4]; size];
            self.fit_closed_1d(size, &intervals, &dependent, &mut work, &mut coefficients);

            self.base.set_intervals(intervals);
            self.base.set_coefficients(coefficients);
        }

        self.base.set_compute_time_to_m_time();
    }

    /// VTK: `vtkCardinalSpline::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.base.deep_copy(&other.base);
    }

    /// VTK: `vtkCardinalSpline::Fit1D`.
    pub(crate) fn fit_1d(
        &self,
        size: usize,
        x: &[f64],
        y: &[f64],
        work: &mut [f64],
        coefficients: &mut [[f64; 4]],
        left_constraint: i32,
        left_value: f64,
        right_constraint: i32,
        right_value: f64,
    ) {
        match left_constraint {
            0 => {
                coefficients[0][1] = 1.0;
                coefficients[0][2] = 0.0;
                work[0] = self.base.compute_left_derivative();
            }
            1 => {
                coefficients[0][1] = 1.0;
                coefficients[0][2] = 0.0;
                work[0] = left_value;
            }
            2 => {
                coefficients[0][1] = 2.0;
                coefficients[0][2] = 1.0;
                work[0] = 3.0 * ((y[1] - y[0]) / (x[1] - x[0])) - 0.5 * (x[1] - x[0]) * left_value;
            }
            3 => {
                coefficients[0][1] = 2.0;
                coefficients[0][2] = 4.0 * ((0.5 + left_value) / (2.0 + left_value));
                work[0] = 6.0
                    * ((1.0 + left_value) / (2.0 + left_value))
                    * ((y[1] - y[0]) / (x[1] - x[0]));
            }
            _ => unreachable!("vtkSpline clamps LeftConstraint into [0, 3]"),
        }

        for k in 1..size - 1 {
            let xlk = x[k] - x[k - 1];
            let xlkp = x[k + 1] - x[k];
            coefficients[k][0] = xlkp;
            coefficients[k][1] = 2.0 * (xlkp + xlk);
            coefficients[k][2] = xlk;
            work[k] =
                3.0 * (((xlkp * (y[k] - y[k - 1])) / xlk) + ((xlk * (y[k + 1] - y[k])) / xlkp));
        }

        match right_constraint {
            0 => {
                coefficients[size - 1][0] = 0.0;
                coefficients[size - 1][1] = 1.0;
                work[size - 1] = self.base.compute_right_derivative();
            }
            1 => {
                coefficients[size - 1][0] = 0.0;
                coefficients[size - 1][1] = 1.0;
                work[size - 1] = right_value;
            }
            2 => {
                coefficients[size - 1][0] = 1.0;
                coefficients[size - 1][1] = 2.0;
                work[size - 1] = 3.0 * ((y[size - 1] - y[size - 2]) / (x[size - 1] - x[size - 2]))
                    + 0.5 * (x[size - 1] - x[size - 2]) * right_value;
            }
            3 => {
                coefficients[size - 1][0] = 4.0 * ((0.5 + right_value) / (2.0 + right_value));
                coefficients[size - 1][1] = 2.0;
                work[size - 1] = 6.0
                    * ((1.0 + right_value) / (2.0 + right_value))
                    * ((y[size - 1] - y[size - 2]) / (x[size - 1] - x[size - 2]));
            }
            _ => unreachable!("vtkSpline clamps RightConstraint into [0, 3]"),
        }

        coefficients[0][2] /= coefficients[0][1];
        work[0] /= coefficients[0][1];
        coefficients[size - 1][2] = 0.0;

        for k in 1..size {
            coefficients[k][1] -= coefficients[k][0] * coefficients[k - 1][2];
            coefficients[k][2] /= coefficients[k][1];
            work[k] = (work[k] - coefficients[k][0] * work[k - 1]) / coefficients[k][1];
        }

        for k in (0..size - 1).rev() {
            work[k] -= coefficients[k][2] * work[k + 1];
        }

        let mut b = 0.0;
        for k in 0..size - 1 {
            b = x[k + 1] - x[k];
            coefficients[k][0] = y[k];
            coefficients[k][1] = work[k];
            coefficients[k][2] =
                (3.0 * (y[k + 1] - y[k])) / (b * b) - (work[k + 1] + 2.0 * work[k]) / b;
            coefficients[k][3] =
                (2.0 * (y[k] - y[k + 1])) / (b * b * b) + (work[k + 1] + work[k]) / (b * b);
        }

        coefficients[size - 1][0] = y[size - 1];
        coefficients[size - 1][1] = work[size - 1];
        coefficients[size - 1][2] = coefficients[size - 2][2] + 3.0 * coefficients[size - 2][3] * b;
        coefficients[size - 1][3] = coefficients[size - 2][3];
    }

    /// VTK: `vtkCardinalSpline::FitClosed1D`.
    pub(crate) fn fit_closed_1d(
        &self,
        size: usize,
        x: &[f64],
        y: &[f64],
        work: &mut [f64],
        coefficients: &mut [[f64; 4]],
    ) {
        let n = size - 1;

        for k in 1..n {
            let xlk = x[k] - x[k - 1];
            let xlkp = x[k + 1] - x[k];
            coefficients[k][0] = xlkp;
            coefficients[k][1] = 2.0 * (xlkp + xlk);
            coefficients[k][2] = xlk;
            work[k] =
                3.0 * (((xlkp * (y[k] - y[k - 1])) / xlk) + ((xlk * (y[k + 1] - y[k])) / xlkp));
        }

        let xlk = x[n] - x[n - 1];
        let xlkp = x[1] - x[0];
        coefficients[n][0] = xlkp;
        let a_n = coefficients[n][0];
        coefficients[n][1] = 2.0 * (xlkp + xlk);
        let b_n = coefficients[n][1];
        coefficients[n][2] = xlk;
        let c_n = coefficients[n][2];
        work[n] = 3.0 * (((xlkp * (y[n] - y[n - 1])) / xlk) + ((xlk * (y[1] - y[0])) / xlkp));
        let d_n = work[n];

        coefficients[0][2] = 0.0;
        work[0] = 0.0;
        coefficients[0][3] = 1.0;

        for k in 1..=n {
            coefficients[k][1] -= coefficients[k][0] * coefficients[k - 1][2];
            coefficients[k][2] /= coefficients[k][1];
            work[k] = (work[k] - coefficients[k][0] * work[k - 1]) / coefficients[k][1];
            coefficients[k][3] =
                (-coefficients[k][0] * coefficients[k - 1][3]) / coefficients[k][1];
        }

        coefficients[n][0] = 1.0;
        coefficients[n][1] = 0.0;

        for k in (1..n).rev() {
            coefficients[k][0] = coefficients[k][3] - coefficients[k][2] * coefficients[k + 1][0];
            coefficients[k][1] = work[k] - coefficients[k][2] * coefficients[k + 1][1];
        }

        work[0] = (d_n - c_n * coefficients[1][1] - a_n * coefficients[n - 1][1])
            / (b_n + c_n * coefficients[1][0] + a_n * coefficients[n - 1][0]);
        work[n] = work[0];

        for k in 1..n {
            work[k] = coefficients[k][0] * work[n] + coefficients[k][1];
        }

        for k in 0..n {
            let b = x[k + 1] - x[k];
            coefficients[k][0] = y[k];
            coefficients[k][1] = work[k];
            coefficients[k][2] =
                (3.0 * (y[k + 1] - y[k])) / (b * b) - (work[k + 1] + 2.0 * work[k]) / b;
            coefficients[k][3] =
                (2.0 * (y[k] - y[k + 1])) / (b * b * b) + (work[k + 1] + work[k]) / (b * b);
        }

        coefficients[n][0] = y[n];
        coefficients[n][1] = work[n];
        coefficients[n][2] = coefficients[0][2];
        coefficients[n][3] = coefficients[0][3];
    }
}

impl Default for CardinalSpline {
    fn default() -> Self {
        Self::new()
    }
}

impl SplineApi for CardinalSpline {
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
        CardinalSpline::compute(self);
    }

    fn evaluate(&mut self, t: f64) -> f64 {
        CardinalSpline::evaluate(self, t)
    }

    fn clone_box(&self) -> Box<dyn SplineApi> {
        Box::new(self.clone())
    }
}
