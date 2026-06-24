use crate::types::BoundingBox;

use crate::data::traits::{DataObject, DataSet};
use crate::data::{DataSetAttributes, FieldData, Points};

/// Curvilinear grid with explicit point coordinates.
///
/// Analogous to VTK's `vtkStructuredGrid`. Points are stored explicitly
/// (unlike ImageData/RectilinearGrid), but the topology is implicitly
/// structured as an i×j×k grid.
#[derive(Debug, Clone)]
pub struct StructuredGrid {
    dimensions: [usize; 3],
    pub points: Points<f64>,
    point_data: DataSetAttributes,
    cell_data: DataSetAttributes,
    field_data: FieldData,
}

impl Default for StructuredGrid {
    fn default() -> Self {
        Self {
            dimensions: [0, 0, 0],
            points: Points::new(),
            point_data: DataSetAttributes::new(),
            cell_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
        }
    }
}

impl StructuredGrid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a StructuredGrid with given dimensions and points.
    /// The number of points must equal `nx * ny * nz`.
    pub fn from_dimensions_and_points(dimensions: [usize; 3], points: Points<f64>) -> Self {
        assert_eq!(
            points.len(),
            dimensions[0] * dimensions[1] * dimensions[2],
            "points count must match dimensions product"
        );
        Self {
            dimensions,
            points,
            ..Default::default()
        }
    }

    pub fn dimensions(&self) -> [usize; 3] {
        self.dimensions
    }

    pub fn set_dimensions(&mut self, dims: [usize; 3]) {
        self.dimensions = dims;
    }

    /// Convert a flat point index to (i, j, k).
    pub fn ijk_from_index(&self, idx: usize) -> (usize, usize, usize) {
        let k = idx / (self.dimensions[0] * self.dimensions[1]);
        let remainder = idx % (self.dimensions[0] * self.dimensions[1]);
        let j = remainder / self.dimensions[0];
        let i = remainder % self.dimensions[0];
        (i, j, k)
    }

    /// Convert (i, j, k) to a flat point index.
    pub fn index_from_ijk(&self, i: usize, j: usize, k: usize) -> usize {
        k * self.dimensions[0] * self.dimensions[1] + j * self.dimensions[0] + i
    }

    pub fn point_data(&self) -> &DataSetAttributes {
        &self.point_data
    }

    pub fn point_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.point_data
    }

    pub fn cell_data(&self) -> &DataSetAttributes {
        &self.cell_data
    }

    pub fn cell_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.cell_data
    }

    /// Create a structured grid from a function that maps (i,j,k) to position.
    pub fn from_function(dims: [usize; 3], f: impl Fn(usize, usize, usize) -> [f64; 3]) -> Self {
        let mut pts = Points::new();
        for k in 0..dims[2] {
            for j in 0..dims[1] {
                for i in 0..dims[0] {
                    pts.push(f(i, j, k));
                }
            }
        }
        Self::from_dimensions_and_points(dims, pts)
    }

    /// Create a uniform structured grid (same as ImageData geometry but explicit).
    pub fn uniform(dims: [usize; 3], spacing: [f64; 3], origin: [f64; 3]) -> Self {
        Self::from_function(dims, |i, j, k| {
            [
                origin[0] + i as f64 * spacing[0],
                origin[1] + j as f64 * spacing[1],
                origin[2] + k as f64 * spacing[2],
            ]
        })
    }

    /// Builder: add point data.
    pub fn with_point_array(mut self, array: crate::data::AnyDataArray) -> Self {
        let name = array.name().to_string();
        self.point_data.add_array(array);
        if self.point_data.scalars().is_none() {
            self.point_data.set_active_scalars(&name);
        }
        self
    }
}

impl std::fmt::Display for StructuredGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.dimensions;
        write!(
            f,
            "StructuredGrid: {}x{}x{}, {} points, {} point arrays",
            d[0],
            d[1],
            d[2],
            self.points.len(),
            self.point_data.num_arrays()
        )
    }
}

impl DataObject for StructuredGrid {
    fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

impl DataSet for StructuredGrid {
    fn num_points(&self) -> usize {
        self.points.len()
    }

    fn num_cells(&self) -> usize {
        let d = self.dimensions;
        if d.contains(&0) {
            return 0;
        }
        let cx = d[0].saturating_sub(1).max(1);
        let cy = d[1].saturating_sub(1).max(1);
        let cz = d[2].saturating_sub(1).max(1);
        cx * cy * cz
    }

    fn point(&self, idx: usize) -> [f64; 3] {
        self.points.get(idx)
    }

    fn bounds(&self) -> BoundingBox {
        self.points.bounds()
    }

    fn point_data(&self) -> &DataSetAttributes {
        &self.point_data
    }

    fn point_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.point_data
    }

    fn cell_data(&self) -> &DataSetAttributes {
        &self.cell_data
    }

    fn cell_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.cell_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_structured_grid() {
        let mut pts = Points::new();
        // 3x2 grid with some curvature
        for j in 0..2 {
            for i in 0..3 {
                let x = i as f64;
                let y = j as f64 + 0.1 * (i as f64).sin();
                pts.push([x, y, 0.0]);
            }
        }
        let grid = StructuredGrid::from_dimensions_and_points([3, 2, 1], pts);
        assert_eq!(grid.num_points(), 6);
        assert_eq!(grid.num_cells(), 2);
        assert_eq!(grid.dimensions(), [3, 2, 1]);
    }

    #[test]
    fn index_roundtrip() {
        let mut pts = Points::new();
        for _ in 0..24 {
            pts.push([0.0, 0.0, 0.0]);
        }
        let grid = StructuredGrid::from_dimensions_and_points([4, 3, 2], pts);
        for idx in 0..24 {
            let (i, j, k) = grid.ijk_from_index(idx);
            assert_eq!(grid.index_from_ijk(i, j, k), idx);
        }
    }

    #[test]
    fn bounds_from_points() {
        let mut pts = Points::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([10.0, 5.0, 3.0]);
        let grid = StructuredGrid::from_dimensions_and_points([2, 1, 1], pts);
        let bb = grid.bounds();
        assert_eq!(bb.x_max, 10.0);
        assert_eq!(bb.y_max, 5.0);
    }

    #[test]
    fn empty_grid_matches_vtk_defaults() {
        let grid = StructuredGrid::new();
        assert_eq!(grid.dimensions(), [0, 0, 0]);
        assert_eq!(grid.num_points(), 0);
        assert_eq!(grid.num_cells(), 0);
    }

    #[test]
    fn num_points_uses_actual_points() {
        let mut grid = StructuredGrid::new();
        grid.set_dimensions([2, 2, 2]);
        assert_eq!(grid.num_points(), 0);
        assert_eq!(grid.num_cells(), 1);
    }
}
