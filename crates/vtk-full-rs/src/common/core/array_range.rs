use std::fmt;

use super::array_coordinates::CoordinateT;

/// VTK/Common/Core/vtkArrayRange.h
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArrayRange {
    begin: CoordinateT,
    end: CoordinateT,
}

impl ArrayRange {
    /// VTK: vtkArrayRange::vtkArrayRange()
    #[must_use]
    pub fn new() -> Self {
        Self { begin: 0, end: 0 }
    }

    /// VTK: vtkArrayRange::vtkArrayRange(CoordinateT, CoordinateT)
    #[must_use]
    pub fn new_with_range(begin: CoordinateT, end: CoordinateT) -> Self {
        Self {
            begin,
            end: begin.max(end),
        }
    }

    /// VTK: vtkArrayRange::GetBegin()
    #[must_use]
    pub fn get_begin(&self) -> CoordinateT {
        self.begin
    }

    /// VTK: vtkArrayRange::GetEnd()
    #[must_use]
    pub fn get_end(&self) -> CoordinateT {
        self.end
    }

    /// VTK: vtkArrayRange::GetSize()
    #[must_use]
    pub fn get_size(&self) -> CoordinateT {
        self.end - self.begin
    }

    /// VTK: vtkArrayRange::Contains(const vtkArrayRange&) / vtkArrayRange::Contains(CoordinateT)
    #[must_use]
    pub fn contains<T>(&self, value: T) -> bool
    where
        T: ArrayRangeContainsArgument,
    {
        value.contained_by(self)
    }

    pub(crate) fn contains_coordinate(&self, coordinate: CoordinateT) -> bool {
        self.begin <= coordinate && coordinate < self.end
    }

    pub(crate) fn contains_range(&self, range: &Self) -> bool {
        self.begin <= range.begin && range.end <= self.end
    }
}

pub trait ArrayRangeContainsArgument {
    fn contained_by(self, range: &ArrayRange) -> bool;
}

impl ArrayRangeContainsArgument for CoordinateT {
    fn contained_by(self, range: &ArrayRange) -> bool {
        range.contains_coordinate(self)
    }
}

impl ArrayRangeContainsArgument for &ArrayRange {
    fn contained_by(self, range: &ArrayRange) -> bool {
        range.contains_range(self)
    }
}

impl Default for ArrayRange {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ArrayRange {
    /// VTK: operator<<(ostream&, const vtkArrayRange&)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.begin, self.end)
    }
}
