use super::{
    gaussian_random_sequence::GaussianRandomSequence,
    minimal_standard_random_sequence::MinimalStandardRandomSequence,
    random_sequence::RandomSequence, vtk_type::VtkTypeUInt32,
};

/// VTK: `vtkBoxMuellerRandomSequence`.
pub struct BoxMuellerRandomSequence {
    uniform_sequence: Box<dyn RandomSequence>,
    value: f64,
}

impl BoxMuellerRandomSequence {
    /// VTK: `vtkBoxMuellerRandomSequence::New`.
    pub fn new() -> Self {
        Self {
            uniform_sequence: Box::new(MinimalStandardRandomSequence::new()),
            value: 0.0,
        }
    }

    /// VTK: `vtkBoxMuellerRandomSequence::GetUniformSequence`.
    pub fn get_uniform_sequence(&self) -> &dyn RandomSequence {
        self.uniform_sequence.as_ref()
    }

    /// VTK: `vtkBoxMuellerRandomSequence::SetUniformSequence`.
    pub fn set_uniform_sequence(&mut self, uniform_sequence: Box<dyn RandomSequence>) {
        self.uniform_sequence = uniform_sequence;
    }
}

impl Default for BoxMuellerRandomSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSequence for BoxMuellerRandomSequence {
    /// VTK: `vtkBoxMuellerRandomSequence::Initialize`.
    fn initialize(&mut self, _seed: VtkTypeUInt32) {}

    /// VTK: `vtkBoxMuellerRandomSequence::GetValue`.
    fn get_value(&self) -> f64 {
        self.value
    }

    /// VTK: `vtkBoxMuellerRandomSequence::Next`.
    fn next(&mut self) {
        self.uniform_sequence.next();
        let mut x = self.uniform_sequence.get_value();
        while x == 0.0 {
            self.uniform_sequence.next();
            x = self.uniform_sequence.get_value();
        }

        self.uniform_sequence.next();
        let mut y = self.uniform_sequence.get_value();
        while y == 0.0 {
            self.uniform_sequence.next();
            y = self.uniform_sequence.get_value();
        }

        self.value = (-2.0 * x.ln()).sqrt() * (2.0 * std::f64::consts::PI * y).cos();
    }
}

impl GaussianRandomSequence for BoxMuellerRandomSequence {}
