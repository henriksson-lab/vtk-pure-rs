use crate::types::BoundingBox;

use crate::data::traits::{DataObject, DataSet};
use crate::data::{DataSetAttributes, FieldData};

/// Axis-aligned grid with per-axis coordinate arrays.
///
/// Analogous to VTK's `vtkRectilinearGrid`. Unlike `ImageData` where spacing
/// is uniform, a RectilinearGrid allows non-uniform spacing along each axis.
/// Points are implicit — computed from the x, y, z coordinate arrays.
#[derive(Debug, Clone)]
pub struct RectilinearGrid {
    x_coords: Vec<f64>,
    y_coords: Vec<f64>,
    z_coords: Vec<f64>,
    point_data: DataSetAttributes,
    cell_data: DataSetAttributes,
    field_data: FieldData,
}

impl Default for RectilinearGrid {
    fn default() -> Self {
        Self {
            x_coords: vec![0.0],
            y_coords: vec![0.0],
            z_coords: vec![0.0],
            point_data: DataSetAttributes::new(),
            cell_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
        }
    }
}

impl RectilinearGrid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a RectilinearGrid from coordinate arrays.
    pub fn from_coords(x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) -> Self {
        assert!(!x.is_empty() && !y.is_empty() && !z.is_empty());
        Self {
            x_coords: x,
            y_coords: y,
            z_coords: z,
            ..Default::default()
        }
    }

    pub fn x_coords(&self) -> &[f64] {
        &self.x_coords
    }

    pub fn y_coords(&self) -> &[f64] {
        &self.y_coords
    }

    pub fn z_coords(&self) -> &[f64] {
        &self.z_coords
    }

    pub fn set_x_coords(&mut self, coords: Vec<f64>) {
        assert!(!coords.is_empty());
        self.x_coords = coords;
    }

    pub fn set_y_coords(&mut self, coords: Vec<f64>) {
        assert!(!coords.is_empty());
        self.y_coords = coords;
    }

    pub fn set_z_coords(&mut self, coords: Vec<f64>) {
        assert!(!coords.is_empty());
        self.z_coords = coords;
    }

    /// Number of points in each dimension.
    pub fn dimensions(&self) -> [usize; 3] {
        [
            self.x_coords.len(),
            self.y_coords.len(),
            self.z_coords.len(),
        ]
    }

    /// Compute the world-space position of a point given its (i, j, k) index.
    pub fn point_from_ijk(&self, i: usize, j: usize, k: usize) -> [f64; 3] {
        [self.x_coords[i], self.y_coords[j], self.z_coords[k]]
    }

    /// Convert a flat point index to (i, j, k).
    pub fn ijk_from_index(&self, idx: usize) -> (usize, usize, usize) {
        let dims = self.dimensions();
        let k = idx / (dims[0] * dims[1]);
        let remainder = idx % (dims[0] * dims[1]);
        let j = remainder / dims[0];
        let i = remainder % dims[0];
        (i, j, k)
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

    /// Create a uniform RectilinearGrid with n points and spacing in each direction.
    pub fn uniform(nx: usize, ny: usize, nz: usize, spacing: f64) -> Self {
        Self::from_coords(
            (0..nx).map(|i| i as f64 * spacing).collect(),
            (0..ny).map(|i| i as f64 * spacing).collect(),
            (0..nz).map(|i| i as f64 * spacing).collect(),
        )
    }

    /// Builder: add a point data array.
    pub fn with_point_array(mut self, array: crate::data::AnyDataArray) -> Self {
        let name = array.name().to_string();
        self.point_data.add_array(array);
        if self.point_data.scalars().is_none() {
            self.point_data.set_active_scalars(&name);
        }
        self
    }
}

impl std::fmt::Display for RectilinearGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.dimensions();
        write!(
            f,
            "RectilinearGrid: {}x{}x{}, {} point arrays",
            d[0],
            d[1],
            d[2],
            self.point_data.num_arrays()
        )
    }
}

impl DataObject for RectilinearGrid {
    fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

impl DataSet for RectilinearGrid {
    fn num_points(&self) -> usize {
        let d = self.dimensions();
        d[0] * d[1] * d[2]
    }

    fn num_cells(&self) -> usize {
        let d = self.dimensions();
        let cx = d[0].saturating_sub(1);
        let cy = d[1].saturating_sub(1);
        let cz = d[2].saturating_sub(1);
        cx.max(1) * cy.max(1) * cz.max(1)
    }

    fn point(&self, idx: usize) -> [f64; 3] {
        let (i, j, k) = self.ijk_from_index(idx);
        self.point_from_ijk(i, j, k)
    }

    fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        if !self.x_coords.is_empty() && !self.y_coords.is_empty() && !self.z_coords.is_empty() {
            bb.expand([self.x_coords[0], self.y_coords[0], self.z_coords[0]]);
            bb.expand([
                *self.x_coords.last().unwrap(),
                *self.y_coords.last().unwrap(),
                *self.z_coords.last().unwrap(),
            ]);
        }
        bb
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
    fn basic_rectilinear_grid() {
        let grid = RectilinearGrid::from_coords(vec![0.0, 1.0, 3.0], vec![0.0, 2.0], vec![0.0]);
        assert_eq!(grid.dimensions(), [3, 2, 1]);
        assert_eq!(grid.num_points(), 6);
        assert_eq!(grid.num_cells(), 2); // 2x1 cells
    }

    #[test]
    fn point_coordinates() {
        let grid =
            RectilinearGrid::from_coords(vec![0.0, 1.0, 4.0], vec![0.0, 3.0], vec![0.0, 5.0]);
        assert_eq!(grid.point_from_ijk(2, 1, 1), [4.0, 3.0, 5.0]);
    }

    #[test]
    fn index_roundtrip() {
        let grid =
            RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0], vec![0.0, 1.0]);
        for idx in 0..grid.num_points() {
            let (i, j, k) = grid.ijk_from_index(idx);
            let p1 = grid.point_from_ijk(i, j, k);
            let p2 = grid.point(idx);
            assert_eq!(p1, p2);
        }
    }

    #[test]
    fn bounds() {
        let grid = RectilinearGrid::from_coords(
            vec![0.0, 0.5, 1.0, 2.0, 5.0],
            vec![-1.0, 0.0, 3.0],
            vec![0.0, 10.0],
        );
        let bb = grid.bounds();
        assert_eq!(bb.x_min, 0.0);
        assert_eq!(bb.x_max, 5.0);
        assert_eq!(bb.y_min, -1.0);
        assert_eq!(bb.y_max, 3.0);
        assert_eq!(bb.z_min, 0.0);
        assert_eq!(bb.z_max, 10.0);
    }
}
