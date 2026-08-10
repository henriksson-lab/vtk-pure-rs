use std::ops::{Index, IndexMut};

use super::vtk_type::VtkIdType;

/// VTK/Common/Core/vtkArrayWeights.h
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArrayWeights {
    storage: Vec<f64>,
}

impl ArrayWeights {
    /// VTK: vtkArrayWeights::vtkArrayWeights()
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// VTK: vtkArrayWeights::vtkArrayWeights(double)
    #[must_use]
    pub fn new_1(i: f64) -> Self {
        Self { storage: vec![i] }
    }

    /// VTK: vtkArrayWeights::vtkArrayWeights(double, double)
    #[must_use]
    pub fn new_2(i: f64, j: f64) -> Self {
        Self {
            storage: vec![i, j],
        }
    }

    /// VTK: vtkArrayWeights::vtkArrayWeights(double, double, double)
    #[must_use]
    pub fn new_3(i: f64, j: f64, k: f64) -> Self {
        Self {
            storage: vec![i, j, k],
        }
    }

    /// VTK: vtkArrayWeights::vtkArrayWeights(double, double, double, double)
    #[must_use]
    pub fn new_4(i: f64, j: f64, k: f64, l: f64) -> Self {
        Self {
            storage: vec![i, j, k, l],
        }
    }

    /// VTK: vtkArrayWeights::GetCount()
    #[must_use]
    pub fn get_count(&self) -> VtkIdType {
        self.storage.len() as VtkIdType
    }

    /// VTK: vtkArrayWeights::SetCount(vtkIdType)
    pub fn set_count(&mut self, count: VtkIdType) {
        let count = vtk_id_to_usize(count);
        self.storage.clear();
        self.storage.resize(count, 0.0);
    }
}

impl Index<VtkIdType> for ArrayWeights {
    type Output = f64;

    /// VTK: vtkArrayWeights::operator[](vtkIdType) const
    fn index(&self, index: VtkIdType) -> &Self::Output {
        &self.storage[vtk_id_to_usize(index)]
    }
}

impl IndexMut<VtkIdType> for ArrayWeights {
    /// VTK: vtkArrayWeights::operator[](vtkIdType)
    fn index_mut(&mut self, index: VtkIdType) -> &mut Self::Output {
        &mut self.storage[vtk_id_to_usize(index)]
    }
}

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    assert!(
        id >= 0,
        "VTK array weights index/count must be non-negative"
    );
    id as usize
}
