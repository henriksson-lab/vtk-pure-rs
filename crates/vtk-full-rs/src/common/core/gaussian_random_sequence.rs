use super::random_sequence::RandomSequence;

/// VTK: `vtkGaussianRandomSequence`.
pub trait GaussianRandomSequence: RandomSequence {
    /// VTK: `vtkGaussianRandomSequence::GetScaledValue`.
    fn get_scaled_value(&self, mean: f64, standard_deviation: f64) -> f64 {
        mean + standard_deviation * self.get_value()
    }

    /// VTK: `vtkGaussianRandomSequence::GetNextScaledValue`.
    fn get_next_scaled_value(&mut self, mean: f64, standard_deviation: f64) -> f64 {
        self.next();
        self.get_scaled_value(mean, standard_deviation)
    }
}
