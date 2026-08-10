use std::{
    cell::RefCell,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::common::{
    computational_geometry::{ParametricFunction, ParametricFunctionApi},
    core::{
        minimal_standard_random_sequence::MinimalStandardRandomSequence,
        random_sequence::RandomSequence, VtkMTimeType,
    },
};

#[derive(Debug, Clone, PartialEq)]
struct RandomHillsParameters {
    number_of_hills: i32,
    hill_x_variance: f64,
    hill_y_variance: f64,
    hill_amplitude: f64,
    random_seed: i32,
    x_variance_scale_factor: f64,
    y_variance_scale_factor: f64,
    amplitude_scale_factor: f64,
    allow_random_generation: bool,
}

impl Default for RandomHillsParameters {
    fn default() -> Self {
        Self {
            number_of_hills: 0,
            hill_x_variance: 0.0,
            hill_y_variance: 0.0,
            hill_amplitude: 0.0,
            random_seed: 0,
            x_variance_scale_factor: 0.0,
            y_variance_scale_factor: 0.0,
            amplitude_scale_factor: 0.0,
            allow_random_generation: false,
        }
    }
}

/// VTK: `vtkParametricRandomHills`.
#[derive(Debug, Clone)]
pub struct ParametricRandomHills {
    base: ParametricFunction,
    number_of_hills: i32,
    hill_x_variance: f64,
    hill_y_variance: f64,
    hill_amplitude: f64,
    random_seed: i32,
    x_variance_scale_factor: f64,
    y_variance_scale_factor: f64,
    amplitude_scale_factor: f64,
    allow_random_generation: bool,
    previous_parameters: RefCell<RandomHillsParameters>,
    hill_data: RefCell<Vec<[f64; 5]>>,
    random_sequence_generator: RefCell<MinimalStandardRandomSequence>,
}

impl PartialEq for ParametricRandomHills {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
            && self.number_of_hills == other.number_of_hills
            && self.hill_x_variance == other.hill_x_variance
            && self.hill_y_variance == other.hill_y_variance
            && self.hill_amplitude == other.hill_amplitude
            && self.random_seed == other.random_seed
            && self.x_variance_scale_factor == other.x_variance_scale_factor
            && self.y_variance_scale_factor == other.y_variance_scale_factor
            && self.amplitude_scale_factor == other.amplitude_scale_factor
            && self.allow_random_generation == other.allow_random_generation
    }
}

impl ParametricRandomHills {
    /// VTK: `vtkParametricRandomHills::New`.
    pub fn new() -> Self {
        let mut base = ParametricFunction::with_class_name("vtkParametricRandomHills");
        base.set_minimum_u(-10.0);
        base.set_maximum_u(10.0);
        base.set_minimum_v(-10.0);
        base.set_maximum_v(10.0);
        base.set_join_u(false);
        base.set_join_v(false);
        base.set_twist_u(false);
        base.set_twist_v(false);
        base.set_clockwise_ordering(false);
        base.set_derivatives_available(false);

        let mut random_sequence_generator = MinimalStandardRandomSequence::new();
        random_sequence_generator.set_seed(1);

        Self {
            base,
            number_of_hills: 30,
            hill_x_variance: 2.5,
            hill_y_variance: 2.5,
            hill_amplitude: 2.0,
            random_seed: 1,
            x_variance_scale_factor: 1.0 / 3.0,
            y_variance_scale_factor: 1.0 / 3.0,
            amplitude_scale_factor: 1.0 / 3.0,
            allow_random_generation: true,
            previous_parameters: RefCell::new(RandomHillsParameters::default()),
            hill_data: RefCell::new(Vec::new()),
            random_sequence_generator: RefCell::new(random_sequence_generator),
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

    /// VTK: `vtkParametricRandomHills::SetNumberOfHills`.
    pub fn set_number_of_hills(&mut self, value: i32) {
        if self.number_of_hills != value {
            self.number_of_hills = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetNumberOfHills`.
    pub fn get_number_of_hills(&self) -> i32 {
        self.number_of_hills
    }

    /// VTK: `vtkParametricRandomHills::SetHillXVariance`.
    pub fn set_hill_x_variance(&mut self, value: f64) {
        if self.hill_x_variance != value {
            self.hill_x_variance = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetHillXVariance`.
    pub fn get_hill_x_variance(&self) -> f64 {
        self.hill_x_variance
    }

    /// VTK: `vtkParametricRandomHills::SetHillYVariance`.
    pub fn set_hill_y_variance(&mut self, value: f64) {
        if self.hill_y_variance != value {
            self.hill_y_variance = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetHillYVariance`.
    pub fn get_hill_y_variance(&self) -> f64 {
        self.hill_y_variance
    }

    /// VTK: `vtkParametricRandomHills::SetHillAmplitude`.
    pub fn set_hill_amplitude(&mut self, value: f64) {
        if self.hill_amplitude != value {
            self.hill_amplitude = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetHillAmplitude`.
    pub fn get_hill_amplitude(&self) -> f64 {
        self.hill_amplitude
    }

    /// VTK: `vtkParametricRandomHills::SetRandomSeed`.
    pub fn set_random_seed(&mut self, value: i32) {
        if self.random_seed != value {
            self.random_seed = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetRandomSeed`.
    pub fn get_random_seed(&self) -> i32 {
        self.random_seed
    }

    /// VTK: `vtkParametricRandomHills::SetAllowRandomGeneration`.
    pub fn set_allow_random_generation(&mut self, value: bool) {
        if self.allow_random_generation != value {
            self.allow_random_generation = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetAllowRandomGeneration`.
    pub fn get_allow_random_generation(&self) -> bool {
        self.allow_random_generation
    }

    /// VTK: `vtkParametricRandomHills::AllowRandomGenerationOn`.
    pub fn allow_random_generation_on(&mut self) {
        self.set_allow_random_generation(true);
    }

    /// VTK: `vtkParametricRandomHills::AllowRandomGenerationOff`.
    pub fn allow_random_generation_off(&mut self) {
        self.set_allow_random_generation(false);
    }

    /// VTK: `vtkParametricRandomHills::SetXVarianceScaleFactor`.
    pub fn set_x_variance_scale_factor(&mut self, value: f64) {
        if self.x_variance_scale_factor != value {
            self.x_variance_scale_factor = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetXVarianceScaleFactor`.
    pub fn get_x_variance_scale_factor(&self) -> f64 {
        self.x_variance_scale_factor
    }

    /// VTK: `vtkParametricRandomHills::SetYVarianceScaleFactor`.
    pub fn set_y_variance_scale_factor(&mut self, value: f64) {
        if self.y_variance_scale_factor != value {
            self.y_variance_scale_factor = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetYVarianceScaleFactor`.
    pub fn get_y_variance_scale_factor(&self) -> f64 {
        self.y_variance_scale_factor
    }

    /// VTK: `vtkParametricRandomHills::SetAmplitudeScaleFactor`.
    pub fn set_amplitude_scale_factor(&mut self, value: f64) {
        if self.amplitude_scale_factor != value {
            self.amplitude_scale_factor = value;
            self.modified();
        }
    }

    /// VTK: `vtkParametricRandomHills::GetAmplitudeScaleFactor`.
    pub fn get_amplitude_scale_factor(&self) -> f64 {
        self.amplitude_scale_factor
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

    /// VTK: `vtkParametricRandomHills::InitRNG`.
    fn init_rng(&self, random_seed: i32) {
        let seed = if random_seed < 0 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs() as i32)
                .unwrap_or(1)
        } else {
            random_seed
        };
        self.random_sequence_generator.borrow_mut().set_seed(seed);
    }

    /// VTK: `vtkParametricRandomHills::Rand`.
    fn rand(&self) -> f64 {
        let mut rng = self.random_sequence_generator.borrow_mut();
        let value = rng.get_value();
        rng.next();
        value
    }

    /// VTK: `vtkParametricRandomHills::MakeTheHillData`.
    fn make_the_hill_data(&self) {
        let hill_count = self.number_of_hills.max(0) as usize;
        let d_u = self.get_maximum_u() - self.get_minimum_u();
        let d_v = self.get_maximum_v() - self.get_minimum_v();
        let mut hill_data = vec![[0.0; 5]; hill_count];

        if self.allow_random_generation {
            self.init_rng(self.random_seed);
            for hill_tuple in &mut hill_data {
                hill_tuple[0] = self.get_minimum_u() + self.rand() * d_u;
                hill_tuple[1] = self.get_minimum_v() + self.rand() * d_v;
                hill_tuple[2] = self.hill_x_variance * (self.rand() + self.x_variance_scale_factor);
                hill_tuple[3] = self.hill_y_variance * (self.rand() + self.y_variance_scale_factor);
                hill_tuple[4] = self.hill_amplitude * (self.rand() + self.amplitude_scale_factor);
            }
        } else if hill_count > 0 {
            let grid_max = (hill_count as f64).sqrt();
            let grid_count = grid_max as usize;
            let mut counter = 0usize;
            let mid_u = d_u / 2.0;
            let shift_u = mid_u / grid_max;
            let mid_v = d_v / 2.0;
            let shift_v = mid_v / grid_max;

            let fixed_var_x = self.hill_x_variance * self.x_variance_scale_factor;
            let fixed_var_y = self.hill_y_variance * self.y_variance_scale_factor;
            let fixed_amplitude = self.hill_amplitude * self.amplitude_scale_factor;
            for i in 0..grid_count {
                let center_u = self.get_minimum_u() + shift_u + (i as f64 / grid_max) * d_u;
                for j in 0..grid_count {
                    if counter >= hill_data.len() {
                        break;
                    }
                    hill_data[counter] = [
                        center_u,
                        self.get_minimum_v() + shift_v + (j as f64 / grid_max) * d_v,
                        fixed_var_x,
                        fixed_var_y,
                        fixed_amplitude,
                    ];
                    counter += 1;
                }
            }
            for hill_tuple in hill_data.iter_mut().skip(counter) {
                hill_tuple[0] = self.get_minimum_u() + mid_u;
                hill_tuple[1] = self.get_minimum_v() + mid_v;
            }
        }

        *self.hill_data.borrow_mut() = hill_data;
    }

    /// VTK: `vtkParametricRandomHills::ParametersChanged`.
    fn parameters_changed(&self) -> bool {
        let current = self.current_parameters();
        if *self.previous_parameters.borrow() != current {
            self.copy_parameters();
            return true;
        }
        false
    }

    /// VTK: `vtkParametricRandomHills::CopyParameters`.
    fn copy_parameters(&self) {
        *self.previous_parameters.borrow_mut() = self.current_parameters();
    }

    fn current_parameters(&self) -> RandomHillsParameters {
        RandomHillsParameters {
            number_of_hills: self.number_of_hills,
            hill_x_variance: self.hill_x_variance,
            hill_y_variance: self.hill_y_variance,
            hill_amplitude: self.hill_amplitude,
            random_seed: self.random_seed,
            x_variance_scale_factor: self.x_variance_scale_factor,
            y_variance_scale_factor: self.y_variance_scale_factor,
            amplitude_scale_factor: self.amplitude_scale_factor,
            allow_random_generation: self.allow_random_generation,
        }
    }
}

impl ParametricFunctionApi for ParametricRandomHills {
    fn parametric_function(&self) -> &ParametricFunction {
        &self.base
    }

    fn parametric_function_mut(&mut self) -> &mut ParametricFunction {
        &mut self.base
    }

    /// VTK: `vtkParametricRandomHills::GetDimension`.
    fn get_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkParametricRandomHills::Evaluate`.
    fn evaluate(&self, uvw: [f64; 3], pt: &mut [f64; 3], duvw: &mut [f64; 9]) {
        if self.parameters_changed() {
            self.make_the_hill_data();
        }

        let u = uvw[0];
        let v = uvw[1];
        pt.fill(0.0);
        duvw.fill(0.0);

        pt[0] = u;
        pt[1] = self.get_maximum_v() - v;
        for hill_tuple in self.hill_data.borrow().iter() {
            let x = (u - hill_tuple[0]) / hill_tuple[2];
            let y = (v - hill_tuple[1]) / hill_tuple[3];
            pt[2] += hill_tuple[4] * (-(x * x + y * y) / 2.0).exp();
        }
    }

    /// VTK: `vtkParametricRandomHills::EvaluateScalar`.
    fn evaluate_scalar(&self, _uvw: [f64; 3], _pt: [f64; 3], _duvw: [f64; 9]) -> f64 {
        0.0
    }
}

impl Default for ParametricRandomHills {
    fn default() -> Self {
        Self::new()
    }
}
