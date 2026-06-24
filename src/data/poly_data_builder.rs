use crate::data::{CellArray, Points, PolyData};

/// Fluent builder for constructing PolyData incrementally.
///
/// ```
/// use vtk_pure_rs::data::PolyDataBuilder;
///
/// let pd = PolyDataBuilder::new()
///     .point([0.0, 0.0, 0.0])
///     .point([1.0, 0.0, 0.0])
///     .point([0.0, 1.0, 0.0])
///     .triangle(0, 1, 2)
///     .build();
/// assert_eq!(pd.points.len(), 3);
/// assert_eq!(pd.polys.num_cells(), 1);
/// ```
pub struct PolyDataBuilder {
    points: Points<f64>,
    polys: CellArray,
    lines: CellArray,
    verts: CellArray,
}

impl PolyDataBuilder {
    pub fn new() -> Self {
        Self {
            points: Points::new(),
            polys: CellArray::new(),
            lines: CellArray::new(),
            verts: CellArray::new(),
        }
    }

    /// Add a point, returns self for chaining.
    pub fn point(mut self, p: [f64; 3]) -> Self {
        self.points.push(p);
        self
    }

    /// Add multiple points at once.
    pub fn points(mut self, pts: &[[f64; 3]]) -> Self {
        for p in pts {
            self.points.push(*p);
        }
        self
    }

    /// Add a triangle cell.
    pub fn triangle(mut self, i0: i64, i1: i64, i2: i64) -> Self {
        self.polys.push_cell(&[i0, i1, i2]);
        self
    }

    /// Add a quad cell.
    pub fn quad(mut self, i0: i64, i1: i64, i2: i64, i3: i64) -> Self {
        self.polys.push_cell(&[i0, i1, i2, i3]);
        self
    }

    /// Add a polygon cell with arbitrary number of vertices.
    pub fn polygon(mut self, ids: &[i64]) -> Self {
        self.polys.push_cell(ids);
        self
    }

    /// Add a line cell.
    pub fn line(mut self, i0: i64, i1: i64) -> Self {
        self.lines.push_cell(&[i0, i1]);
        self
    }

    /// Add a vertex cell.
    pub fn vertex(mut self, id: i64) -> Self {
        self.verts.push_cell(&[id]);
        self
    }

    /// Get the current number of points.
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    /// Build the final PolyData.
    pub fn build(self) -> PolyData {
        let mut pd = PolyData::new();
        pd.points = self.points;
        pd.polys = self.polys;
        pd.lines = self.lines;
        pd.verts = self.verts;
        pd
    }
}

impl Default for PolyDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_triangle() {
        let pd = PolyDataBuilder::new()
            .point([0.0, 0.0, 0.0])
            .point([1.0, 0.0, 0.0])
            .point([0.0, 1.0, 0.0])
            .triangle(0, 1, 2)
            .build();
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.polys.num_cells(), 1);
    }

    #[test]
    fn build_mixed() {
        let pd = PolyDataBuilder::new()
            .points(&[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ])
            .triangle(0, 1, 2)
            .quad(0, 1, 3, 2)
            .line(0, 1)
            .vertex(3)
            .build();
        assert_eq!(pd.points.len(), 4);
        assert_eq!(pd.polys.num_cells(), 2);
        assert_eq!(pd.lines.num_cells(), 1);
        assert_eq!(pd.verts.num_cells(), 1);
    }

    #[test]
    fn num_points_during_build() {
        let builder = PolyDataBuilder::new()
            .point([0.0; 3])
            .point([1.0, 0.0, 0.0]);
        assert_eq!(builder.num_points(), 2);
    }
}
