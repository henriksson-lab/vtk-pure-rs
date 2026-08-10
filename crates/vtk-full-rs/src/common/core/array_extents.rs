use std::fmt;
use std::ops::{Index, IndexMut};

use super::array_coordinates::{ArrayCoordinates, CoordinateT, DimensionT};
use super::array_range::ArrayRange;

pub type SizeT = u64;

/// VTK/Common/Core/vtkArrayExtents.h
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArrayExtents {
    storage: Vec<ArrayRange>,
}

impl ArrayExtents {
    /// VTK: vtkArrayExtents::vtkArrayExtents()
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// VTK: vtkArrayExtents::vtkArrayExtents(CoordinateT)
    #[must_use]
    pub fn new_1d(i: CoordinateT) -> Self {
        Self {
            storage: vec![ArrayRange::new_with_range(0, i)],
        }
    }

    /// VTK: vtkArrayExtents::vtkArrayExtents(const vtkArrayRange&)
    #[must_use]
    pub fn new_1d_range(i: ArrayRange) -> Self {
        Self { storage: vec![i] }
    }

    /// VTK: vtkArrayExtents::vtkArrayExtents(CoordinateT, CoordinateT)
    #[must_use]
    pub fn new_2d(i: CoordinateT, j: CoordinateT) -> Self {
        Self {
            storage: vec![
                ArrayRange::new_with_range(0, i),
                ArrayRange::new_with_range(0, j),
            ],
        }
    }

    /// VTK: vtkArrayExtents::vtkArrayExtents(const vtkArrayRange&, const vtkArrayRange&)
    #[must_use]
    pub fn new_2d_range(i: ArrayRange, j: ArrayRange) -> Self {
        Self {
            storage: vec![i, j],
        }
    }

    /// VTK: vtkArrayExtents::vtkArrayExtents(CoordinateT, CoordinateT, CoordinateT)
    #[must_use]
    pub fn new_3d(i: CoordinateT, j: CoordinateT, k: CoordinateT) -> Self {
        Self {
            storage: vec![
                ArrayRange::new_with_range(0, i),
                ArrayRange::new_with_range(0, j),
                ArrayRange::new_with_range(0, k),
            ],
        }
    }

    /// VTK: vtkArrayExtents::vtkArrayExtents(const vtkArrayRange&, const vtkArrayRange&, const vtkArrayRange&)
    #[must_use]
    pub fn new_3d_range(i: ArrayRange, j: ArrayRange, k: ArrayRange) -> Self {
        Self {
            storage: vec![i, j, k],
        }
    }

    /// VTK: vtkArrayExtents::Uniform(DimensionT, CoordinateT)
    #[must_use]
    pub fn uniform(n: DimensionT, m: CoordinateT) -> Self {
        let n = dimension_to_usize(n);
        Self {
            storage: vec![ArrayRange::new_with_range(0, m); n],
        }
    }

    /// VTK: vtkArrayExtents::Append(const vtkArrayRange&)
    pub fn append(&mut self, extent: ArrayRange) {
        self.storage.push(extent);
    }

    /// VTK: vtkArrayExtents::GetDimensions()
    #[must_use]
    pub fn get_dimensions(&self) -> DimensionT {
        self.storage.len() as DimensionT
    }

    /// VTK: vtkArrayExtents::GetSize()
    #[must_use]
    pub fn get_size(&self) -> SizeT {
        if self.storage.is_empty() {
            return 0;
        }

        self.storage.iter().fold(1_u64, |size, extent| {
            let extent_size = extent.get_size();
            assert!(
                extent_size >= 0,
                "VTK array extent size must be non-negative"
            );
            size.wrapping_mul(extent_size as u64)
        })
    }

    /// VTK: vtkArrayExtents::SetDimensions(DimensionT)
    pub fn set_dimensions(&mut self, dimensions: DimensionT) {
        let dimensions = dimension_to_usize(dimensions);
        self.storage.clear();
        self.storage.resize(dimensions, ArrayRange::new());
    }

    /// VTK: vtkArrayExtents::GetExtent(DimensionT)
    #[must_use]
    pub fn get_extent(&self, i: DimensionT) -> ArrayRange {
        self[i]
    }

    /// VTK: vtkArrayExtents::SetExtent(DimensionT, const vtkArrayRange&)
    pub fn set_extent(&mut self, i: DimensionT, extent: ArrayRange) {
        self[i] = extent;
    }

    /// VTK: vtkArrayExtents::ZeroBased()
    #[must_use]
    pub fn zero_based(&self) -> bool {
        self.storage.iter().all(|extent| extent.get_begin() == 0)
    }

    /// VTK: vtkArrayExtents::SameShape(const vtkArrayExtents&)
    #[must_use]
    pub fn same_shape(&self, rhs: &Self) -> bool {
        self.get_dimensions() == rhs.get_dimensions()
            && self
                .storage
                .iter()
                .zip(rhs.storage.iter())
                .all(|(lhs, rhs)| lhs.get_size() == rhs.get_size())
    }

    /// VTK: vtkArrayExtents::GetLeftToRightCoordinatesN(SizeT, vtkArrayCoordinates&)
    pub fn get_left_to_right_coordinates_n(&self, n: SizeT, coordinates: &mut ArrayCoordinates) {
        coordinates.set_dimensions(self.get_dimensions());

        let mut divisor = 1_u64;
        for i in 0..self.storage.len() {
            let extent = self.storage[i];
            let extent_size = extent_size_u64(extent);
            coordinates[i as DimensionT] =
                ((n / divisor) % extent_size) as CoordinateT + extent.get_begin();
            divisor = divisor.wrapping_mul(extent_size);
        }
    }

    /// VTK: vtkArrayExtents::GetRightToLeftCoordinatesN(SizeT, vtkArrayCoordinates&)
    pub fn get_right_to_left_coordinates_n(&self, n: SizeT, coordinates: &mut ArrayCoordinates) {
        coordinates.set_dimensions(self.get_dimensions());

        let mut divisor = 1_u64;
        for i in (0..self.storage.len()).rev() {
            let extent = self.storage[i];
            let extent_size = extent_size_u64(extent);
            coordinates[i as DimensionT] =
                ((n / divisor) % extent_size) as CoordinateT + extent.get_begin();
            divisor = divisor.wrapping_mul(extent_size);
        }
    }

    /// VTK: vtkArrayExtents::Contains(const vtkArrayExtents&) / vtkArrayExtents::Contains(const vtkArrayCoordinates&)
    #[must_use]
    pub fn contains<T>(&self, value: T) -> bool
    where
        T: ArrayExtentsContainsArgument,
    {
        value.contained_by(self)
    }

    pub(crate) fn contains_extents(&self, extents: &Self) -> bool {
        self.get_dimensions() == extents.get_dimensions()
            && self
                .storage
                .iter()
                .zip(extents.storage.iter())
                .all(|(lhs, rhs)| lhs.contains(rhs))
    }

    pub(crate) fn contains_coordinates(&self, coordinates: &ArrayCoordinates) -> bool {
        self.get_dimensions() == coordinates.get_dimensions()
            && self
                .storage
                .iter()
                .enumerate()
                .all(|(i, extent)| extent.contains(coordinates[i as DimensionT]))
    }
}

pub trait ArrayExtentsContainsArgument {
    fn contained_by(self, extents: &ArrayExtents) -> bool;
}

impl ArrayExtentsContainsArgument for &ArrayExtents {
    fn contained_by(self, extents: &ArrayExtents) -> bool {
        extents.contains_extents(self)
    }
}

impl ArrayExtentsContainsArgument for &ArrayCoordinates {
    fn contained_by(self, extents: &ArrayExtents) -> bool {
        extents.contains_coordinates(self)
    }
}

impl Index<DimensionT> for ArrayExtents {
    type Output = ArrayRange;

    /// VTK: vtkArrayExtents::operator[](DimensionT) const
    fn index(&self, index: DimensionT) -> &Self::Output {
        &self.storage[dimension_to_usize(index)]
    }
}

impl IndexMut<DimensionT> for ArrayExtents {
    /// VTK: vtkArrayExtents::operator[](DimensionT)
    fn index_mut(&mut self, index: DimensionT) -> &mut Self::Output {
        &mut self.storage[dimension_to_usize(index)]
    }
}

impl fmt::Display for ArrayExtents {
    /// VTK: operator<<(ostream&, const vtkArrayExtents&)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, extent) in self.storage.iter().enumerate() {
            if i != 0 {
                f.write_str("x")?;
            }
            write!(f, "[{},{})", extent.get_begin(), extent.get_end())?;
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

fn extent_size_u64(extent: ArrayRange) -> u64 {
    let size = extent.get_size();
    assert!(
        size > 0,
        "VTK coordinate conversion requires positive extents"
    );
    size as u64
}
