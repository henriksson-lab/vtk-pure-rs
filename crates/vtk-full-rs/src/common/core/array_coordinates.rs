use std::fmt;
use std::ops::{Index, IndexMut};

use super::vtk_type::VtkIdType;

pub type CoordinateT = VtkIdType;
pub type DimensionT = VtkIdType;

/// VTK/Common/Core/vtkArrayCoordinates.h
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArrayCoordinates {
    storage: Vec<CoordinateT>,
}

impl ArrayCoordinates {
    /// VTK: vtkArrayCoordinates::vtkArrayCoordinates()
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// VTK: vtkArrayCoordinates::vtkArrayCoordinates(CoordinateT)
    #[must_use]
    pub fn new_1d(i: CoordinateT) -> Self {
        Self { storage: vec![i] }
    }

    /// VTK: vtkArrayCoordinates::vtkArrayCoordinates(CoordinateT, CoordinateT)
    #[must_use]
    pub fn new_2d(i: CoordinateT, j: CoordinateT) -> Self {
        Self {
            storage: vec![i, j],
        }
    }

    /// VTK: vtkArrayCoordinates::vtkArrayCoordinates(CoordinateT, CoordinateT, CoordinateT)
    #[must_use]
    pub fn new_3d(i: CoordinateT, j: CoordinateT, k: CoordinateT) -> Self {
        Self {
            storage: vec![i, j, k],
        }
    }

    /// VTK: vtkArrayCoordinates::GetDimensions()
    #[must_use]
    pub fn get_dimensions(&self) -> DimensionT {
        self.storage.len() as DimensionT
    }

    /// VTK: vtkArrayCoordinates::SetDimensions(DimensionT)
    pub fn set_dimensions(&mut self, dimensions: DimensionT) {
        let dimensions = dimension_to_usize(dimensions);
        self.storage.clear();
        self.storage.resize(dimensions, 0);
    }

    /// VTK: vtkArrayCoordinates::GetCoordinate(DimensionT)
    #[must_use]
    pub fn get_coordinate(&self, i: DimensionT) -> CoordinateT {
        self[i]
    }

    /// VTK: vtkArrayCoordinates::SetCoordinate(DimensionT, const CoordinateT&)
    pub fn set_coordinate(&mut self, i: DimensionT, coordinate: CoordinateT) {
        self[i] = coordinate;
    }
}

impl Index<DimensionT> for ArrayCoordinates {
    type Output = CoordinateT;

    /// VTK: vtkArrayCoordinates::operator[](DimensionT) const
    fn index(&self, index: DimensionT) -> &Self::Output {
        &self.storage[dimension_to_usize(index)]
    }
}

impl IndexMut<DimensionT> for ArrayCoordinates {
    /// VTK: vtkArrayCoordinates::operator[](DimensionT)
    fn index_mut(&mut self, index: DimensionT) -> &mut Self::Output {
        &mut self.storage[dimension_to_usize(index)]
    }
}

impl fmt::Display for ArrayCoordinates {
    /// VTK: operator<<(ostream&, const vtkArrayCoordinates&)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, coordinate) in self.storage.iter().enumerate() {
            if i != 0 {
                f.write_str(",")?;
            }
            write!(f, "{coordinate}")?;
        }
        Ok(())
    }
}

fn dimension_to_usize(dimension: DimensionT) -> usize {
    assert!(
        dimension >= 0,
        "VTK array dimension index must be non-negative"
    );
    dimension as usize
}
