use crate::types::{BoundingBox, CellType};

use crate::data::traits::{DataObject, DataSet};
use crate::data::{CellArray, DataSetAttributes, FieldData, Points};

/// Arbitrary mixed-cell mesh with explicit point coordinates and cell connectivity.
///
/// Analogous to VTK's `vtkUnstructuredGrid`. Each cell has a type (from `CellType`)
/// and point connectivity stored in a shared `CellArray`. Cell types are stored in
/// a parallel array.
#[derive(Debug, Clone, Default)]
pub struct UnstructuredGrid {
    pub points: Points<f64>,
    cells: CellArray,
    cell_types: Vec<CellType>,
    point_data: DataSetAttributes,
    cell_data: DataSetAttributes,
    field_data: FieldData,
}

impl UnstructuredGrid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a cell with the given type and point indices.
    pub fn push_cell(&mut self, cell_type: CellType, point_ids: &[i64]) {
        self.cells.push_cell(point_ids);
        self.cell_types.push(cell_type);
    }

    /// Get the type of cell at the given index.
    pub fn cell_type(&self, idx: usize) -> CellType {
        self.cell_types[idx]
    }

    /// Get the point indices for cell at the given index.
    pub fn cell_points(&self, idx: usize) -> &[i64] {
        self.cells.cell(idx)
    }

    /// Get all cell types.
    pub fn cell_types(&self) -> &[CellType] {
        &self.cell_types
    }

    /// Get the underlying cell array.
    pub fn cells(&self) -> &CellArray {
        &self.cells
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

    /// Create from points and tetrahedra.
    pub fn from_tetrahedra(points: Vec<[f64; 3]>, tets: Vec<[i64; 4]>) -> Self {
        let mut ug = Self::new();
        ug.points = Points::from_vec(points);
        for tet in &tets {
            ug.push_cell(CellType::Tetra, tet);
        }
        ug
    }

    /// Create from points and hexahedra.
    pub fn from_hexahedra(points: Vec<[f64; 3]>, hexes: Vec<[i64; 8]>) -> Self {
        let mut ug = Self::new();
        ug.points = Points::from_vec(points);
        for hex in &hexes {
            ug.push_cell(CellType::Hexahedron, hex);
        }
        ug
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

    /// Builder: add a cell data array.
    pub fn with_cell_array(mut self, array: crate::data::AnyDataArray) -> Self {
        self.cell_data.add_array(array);
        self
    }
}

impl DataObject for UnstructuredGrid {
    fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

impl DataSet for UnstructuredGrid {
    fn num_points(&self) -> usize {
        self.points.len()
    }

    fn num_cells(&self) -> usize {
        self.cells.num_cells()
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

impl std::fmt::Display for UnstructuredGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UnstructuredGrid: {} points, {} cells, {} point arrays",
            self.points.len(),
            self.cells.num_cells(),
            self.point_data.num_arrays()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tetrahedron() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);

        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        assert_eq!(grid.num_points(), 4);
        assert_eq!(grid.num_cells(), 1);
        assert_eq!(grid.cell_type(0), CellType::Tetra);
        assert_eq!(grid.cell_points(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn mixed_cells() {
        let mut grid = UnstructuredGrid::new();
        for i in 0..8 {
            let x = (i % 2) as f64;
            let y = ((i / 2) % 2) as f64;
            let z = (i / 4) as f64;
            grid.points.push([x, y, z]);
        }

        // A hexahedron
        grid.push_cell(CellType::Hexahedron, &[0, 1, 3, 2, 4, 5, 7, 6]);
        // A triangle on the top face
        grid.push_cell(CellType::Triangle, &[4, 5, 7]);

        assert_eq!(grid.num_cells(), 2);
        assert_eq!(grid.cell_type(0), CellType::Hexahedron);
        assert_eq!(grid.cell_type(1), CellType::Triangle);
    }
}
