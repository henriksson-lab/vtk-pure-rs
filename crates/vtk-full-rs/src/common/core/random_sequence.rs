use super::vtk_type::VtkTypeUInt32;

/// VTK `vtkRandomSequence`.
pub trait RandomSequence {
    /// VTK: `vtkRandomSequence::Initialize`.
    fn initialize(&mut self, seed: VtkTypeUInt32);

    /// VTK: `vtkRandomSequence::GetValue`.
    fn get_value(&self) -> f64;

    /// VTK: `vtkRandomSequence::Next`.
    fn next(&mut self);

    /// VTK: `vtkRandomSequence::GetNextValue`.
    fn get_next_value(&mut self) -> f64 {
        self.next();
        self.get_value()
    }
}
