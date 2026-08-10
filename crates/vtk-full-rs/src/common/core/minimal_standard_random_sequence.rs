use super::{random_sequence::RandomSequence, vtk_type::VtkTypeUInt32};

const VTK_K_A: i32 = 16807;
const VTK_K_M: i32 = 2147483647;
const VTK_K_Q: i32 = 127773;
const VTK_K_R: i32 = 2836;

/// VTK `vtkMinimalStandardRandomSequence`.
#[derive(Debug, Clone)]
pub struct MinimalStandardRandomSequence {
    seed: i32,
}

impl MinimalStandardRandomSequence {
    /// VTK: `vtkMinimalStandardRandomSequence::New`.
    pub fn new() -> Self {
        Self { seed: 1 }
    }

    /// VTK: `vtkMinimalStandardRandomSequence::SetSeed`.
    pub fn set_seed(&mut self, value: i32) {
        self.set_seed_only(value);
        self.next();
        self.next();
        self.next();
    }

    /// VTK: `vtkMinimalStandardRandomSequence::SetSeedOnly`.
    pub fn set_seed_only(&mut self, value: i32) {
        self.seed = value;
        if self.seed < 1 {
            self.seed += VTK_K_M - 1;
        } else if self.seed == VTK_K_M {
            self.seed = 1;
        }
    }

    /// VTK: `vtkMinimalStandardRandomSequence::GetSeed`.
    pub fn get_seed(&self) -> i32 {
        self.seed
    }

    /// VTK: `vtkMinimalStandardRandomSequence::GetRangeValue`.
    pub fn get_range_value(&self, range_min: f64, range_max: f64) -> f64 {
        if range_min == range_max {
            range_min
        } else {
            range_min + self.get_value() * (range_max - range_min)
        }
    }

    /// VTK: `vtkMinimalStandardRandomSequence::GetNextRangeValue`.
    pub fn get_next_range_value(&mut self, range_min: f64, range_max: f64) -> f64 {
        self.next();
        self.get_range_value(range_min, range_max)
    }
}

impl Default for MinimalStandardRandomSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSequence for MinimalStandardRandomSequence {
    fn initialize(&mut self, seed: VtkTypeUInt32) {
        self.set_seed(seed as i32);
    }

    fn get_value(&self) -> f64 {
        self.seed as f64 / VTK_K_M as f64
    }

    fn next(&mut self) {
        let hi = self.seed / VTK_K_Q;
        let lo = self.seed % VTK_K_Q;
        self.seed = VTK_K_A * lo - VTK_K_R * hi;
        if self.seed <= 0 {
            self.seed += VTK_K_M;
        }
    }
}
