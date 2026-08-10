use std::fmt;
use std::ops::{Index, IndexMut};

use super::array_coordinates::DimensionT;

/// VTK/Common/Core/vtkArraySort.h
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArraySort {
    storage: Vec<DimensionT>,
}

impl ArraySort {
    /// VTK: vtkArraySort::vtkArraySort()
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// VTK: vtkArraySort::vtkArraySort(DimensionT)
    #[must_use]
    pub fn new_1d(i: DimensionT) -> Self {
        Self { storage: vec![i] }
    }

    /// VTK: vtkArraySort::vtkArraySort(DimensionT, DimensionT)
    #[must_use]
    pub fn new_2d(i: DimensionT, j: DimensionT) -> Self {
        Self {
            storage: vec![i, j],
        }
    }

    /// VTK: vtkArraySort::vtkArraySort(DimensionT, DimensionT, DimensionT)
    #[must_use]
    pub fn new_3d(i: DimensionT, j: DimensionT, k: DimensionT) -> Self {
        Self {
            storage: vec![i, j, k],
        }
    }

    /// VTK: vtkArraySort::GetDimensions()
    #[must_use]
    pub fn get_dimensions(&self) -> DimensionT {
        self.storage.len() as DimensionT
    }

    /// VTK: vtkArraySort::SetDimensions(DimensionT)
    pub fn set_dimensions(&mut self, dimensions: DimensionT) {
        let dimensions = dimension_to_usize(dimensions);
        self.storage.clear();
        self.storage.resize(dimensions, 0);
    }
}

impl Index<DimensionT> for ArraySort {
    type Output = DimensionT;

    /// VTK: vtkArraySort::operator[](DimensionT) const
    fn index(&self, index: DimensionT) -> &Self::Output {
        &self.storage[dimension_to_usize(index)]
    }
}

impl IndexMut<DimensionT> for ArraySort {
    /// VTK: vtkArraySort::operator[](DimensionT)
    fn index_mut(&mut self, index: DimensionT) -> &mut Self::Output {
        &mut self.storage[dimension_to_usize(index)]
    }
}

impl fmt::Display for ArraySort {
    /// VTK: operator<<(ostream&, const vtkArraySort&)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, dimension) in self.storage.iter().enumerate() {
            if i != 0 {
                f.write_str(",")?;
            }
            write!(f, "{dimension}")?;
        }
        Ok(())
    }
}

fn dimension_to_usize(dimension: DimensionT) -> usize {
    assert!(
        dimension >= 0,
        "VTK array sort dimension must be non-negative"
    );
    dimension as usize
}
