use crate::data::{DataSetAttributes, FieldData, Points};
use crate::types::{BoundingBox, VtkError};

/// An explicit structured grid with blanking support.
///
/// Like `StructuredGrid`, this has a structured i x j x k topology,
/// but additionally supports cell blanking (hiding individual cells)
/// via a visibility array. This is analogous to VTK's `vtkExplicitStructuredGrid`.
///
/// Points are stored explicitly (not implicit like ImageData).
/// Cells can be individually blanked (hidden) for AMR-like hole cutting.
#[derive(Debug, Clone)]
pub struct ExplicitStructuredGrid {
    /// Grid dimensions [ni, nj, nk] (number of points along each axis).
    dimensions: [usize; 3],
    /// Explicit point coordinates.
    pub points: Points<f64>,
    /// Explicit hexahedron connectivity. Length = (ni-1)*(nj-1)*(nk-1), 8 point ids per cell.
    cell_points: Vec<[usize; 8]>,
    /// Cell blanking: true = visible, false = blanked. Length = (ni-1)*(nj-1)*(nk-1).
    cell_visibility: Vec<bool>,
    /// Point data attributes.
    point_data: DataSetAttributes,
    /// Cell data attributes.
    cell_data: DataSetAttributes,
    /// Field data (metadata).
    field_data: FieldData,
}

impl ExplicitStructuredGrid {
    /// Create a new explicit structured grid with the given dimensions.
    ///
    /// `dims` is [ni, nj, nk] where ni, nj, nk >= 2.
    /// Points must be provided separately.
    pub fn new(dims: [usize; 3]) -> Self {
        validate_dimensions(dims).expect("explicit structured grid dimensions must all be >= 2");
        let num_cells = checked_num_cells(dims).expect("explicit structured grid is too large");
        Self {
            dimensions: dims,
            points: Points::new(),
            cell_points: default_cell_points(dims),
            cell_visibility: vec![true; num_cells],
            point_data: DataSetAttributes::new(),
            cell_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
        }
    }

    /// Create from dimensions and points.
    pub fn from_points(dims: [usize; 3], points: Points<f64>) -> Result<Self, VtkError> {
        validate_dimensions(dims)?;
        let expected = checked_num_points(dims)?;
        if points.len() != expected {
            return Err(VtkError::InvalidData(format!(
                "expected {} points for {:?} grid, got {}",
                expected,
                dims,
                points.len()
            )));
        }
        let mut grid = Self::new(dims);
        grid.points = points;
        Ok(grid)
    }

    /// Grid dimensions [ni, nj, nk].
    pub fn dimensions(&self) -> [usize; 3] {
        self.dimensions
    }

    /// Number of points.
    pub fn num_points(&self) -> usize {
        checked_num_points(self.dimensions).expect("validated dimensions overflowed")
    }

    /// Number of cells (including blanked).
    pub fn num_cells(&self) -> usize {
        checked_num_cells(self.dimensions).expect("validated dimensions overflowed")
    }

    /// Number of visible (non-blanked) cells.
    pub fn num_visible_cells(&self) -> usize {
        self.cell_visibility.iter().filter(|&&v| v).count()
    }

    /// Blank (hide) a cell by its structured index (i, j, k).
    pub fn blank_cell(&mut self, i: usize, j: usize, k: usize) {
        if let Some(idx) = self.checked_cell_index(i, j, k) {
            self.cell_visibility[idx] = false;
        }
    }

    /// Unblank (show) a cell.
    pub fn unblank_cell(&mut self, i: usize, j: usize, k: usize) {
        if let Some(idx) = self.checked_cell_index(i, j, k) {
            self.cell_visibility[idx] = true;
        }
    }

    /// Check if a cell is visible.
    pub fn is_cell_visible(&self, i: usize, j: usize, k: usize) -> bool {
        self.checked_cell_index(i, j, k)
            .is_some_and(|idx| self.cell_visibility[idx])
    }

    /// Get cell visibility array.
    pub fn cell_visibility(&self) -> &[bool] {
        &self.cell_visibility
    }

    /// Get the explicit connectivity array.
    pub fn cell_points(&self) -> &[[usize; 8]] {
        &self.cell_points
    }

    /// Replace all explicit cell connectivity.
    pub fn set_cell_points(&mut self, cell_points: Vec<[usize; 8]>) -> Result<(), VtkError> {
        if cell_points.len() != self.num_cells() {
            return Err(VtkError::dim_mismatch(
                self.num_cells().to_string(),
                cell_points.len().to_string(),
            ));
        }
        validate_cell_points(&cell_points, self.num_points())?;
        self.cell_points = cell_points;
        Ok(())
    }

    /// Set the 8 point ids for one cell by its structured index (i, j, k).
    pub fn set_cell_point_ids(
        &mut self,
        i: usize,
        j: usize,
        k: usize,
        point_ids: [usize; 8],
    ) -> Result<(), VtkError> {
        let idx = self.checked_cell_index(i, j, k).ok_or_else(|| {
            VtkError::index_oob(self.linear_cell_offset(i, j, k), self.num_cells())
        })?;
        validate_cell_points(&[point_ids], self.num_points())?;
        self.cell_points[idx] = point_ids;
        Ok(())
    }

    /// Set cell visibility array.
    pub fn set_cell_visibility(&mut self, vis: Vec<bool>) -> Result<(), VtkError> {
        if vis.len() != self.num_cells() {
            return Err(VtkError::dim_mismatch(
                self.num_cells().to_string(),
                vis.len().to_string(),
            ));
        }
        self.cell_visibility = vis;
        Ok(())
    }

    fn checked_cell_index(&self, i: usize, j: usize, k: usize) -> Option<usize> {
        let ni = self.dimensions[0].saturating_sub(1);
        let nj = self.dimensions[1].saturating_sub(1);
        let nk = self.dimensions[2].saturating_sub(1);
        (i < ni && j < nj && k < nk).then(|| k * ni * nj + j * ni + i)
    }

    fn linear_cell_offset(&self, i: usize, j: usize, k: usize) -> usize {
        let ni = self.dimensions[0].saturating_sub(1);
        let nj = self.dimensions[1].saturating_sub(1);
        k.saturating_mul(ni)
            .saturating_mul(nj)
            .saturating_add(j.saturating_mul(ni))
            .saturating_add(i)
    }

    /// Convert structured cell (i,j,k) to flat cell index.
    pub fn cell_index(&self, i: usize, j: usize, k: usize) -> usize {
        assert!(
            self.checked_cell_index(i, j, k).is_some(),
            "cell index out of range"
        );
        let ni = self.dimensions[0].saturating_sub(1);
        let nj = self.dimensions[1].saturating_sub(1);
        k * ni * nj + j * ni + i
    }

    /// Convert structured point (i,j,k) to flat point index.
    pub fn point_index(&self, i: usize, j: usize, k: usize) -> usize {
        assert!(
            i < self.dimensions[0] && j < self.dimensions[1] && k < self.dimensions[2],
            "point index out of range"
        );
        let ni = self.dimensions[0];
        let nj = self.dimensions[1];
        k * ni * nj + j * ni + i
    }

    /// Get the 8 point indices for a hexahedral cell at (i,j,k).
    pub fn cell_point_ids(&self, i: usize, j: usize, k: usize) -> [usize; 8] {
        let idx = self
            .checked_cell_index(i, j, k)
            .expect("cell index out of range");
        self.cell_points[idx]
    }

    /// Compute axis-aligned bounding box.
    pub fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox {
            x_min: f64::INFINITY,
            x_max: f64::NEG_INFINITY,
            y_min: f64::INFINITY,
            y_max: f64::NEG_INFINITY,
            z_min: f64::INFINITY,
            z_max: f64::NEG_INFINITY,
        };
        for i in 0..self.points.len() {
            let p = self.points.get(i);
            bb.x_min = bb.x_min.min(p[0]);
            bb.x_max = bb.x_max.max(p[0]);
            bb.y_min = bb.y_min.min(p[1]);
            bb.y_max = bb.y_max.max(p[1]);
            bb.z_min = bb.z_min.min(p[2]);
            bb.z_max = bb.z_max.max(p[2]);
        }
        bb
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

    pub fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    pub fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

fn validate_dimensions(dims: [usize; 3]) -> Result<(), VtkError> {
    if dims.iter().all(|&d| d >= 2) {
        Ok(())
    } else {
        Err(VtkError::InvalidData(format!(
            "explicit structured grid dimensions must all be >= 2, got {dims:?}"
        )))
    }
}

fn checked_num_points(dims: [usize; 3]) -> Result<usize, VtkError> {
    dims[0]
        .checked_mul(dims[1])
        .and_then(|n| n.checked_mul(dims[2]))
        .ok_or_else(|| VtkError::InvalidData(format!("grid dimensions too large: {dims:?}")))
}

fn checked_num_cells(dims: [usize; 3]) -> Result<usize, VtkError> {
    dims[0]
        .checked_sub(1)
        .and_then(|ni| dims[1].checked_sub(1).and_then(|nj| ni.checked_mul(nj)))
        .and_then(|n| dims[2].checked_sub(1).and_then(|nk| n.checked_mul(nk)))
        .ok_or_else(|| VtkError::InvalidData(format!("grid dimensions too large: {dims:?}")))
}

fn default_cell_points(dims: [usize; 3]) -> Vec<[usize; 8]> {
    vec![[0; 8]; checked_num_cells(dims).unwrap_or(0)]
}

fn validate_cell_points(cell_points: &[[usize; 8]], num_points: usize) -> Result<(), VtkError> {
    for point_id in cell_points.iter().flatten() {
        if *point_id >= num_points {
            return Err(VtkError::index_oob(*point_id, num_points));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid_2x2x2() -> ExplicitStructuredGrid {
        let dims = [2, 2, 2];
        let mut pts = Points::new();
        for k in 0..2 {
            for j in 0..2 {
                for i in 0..2 {
                    pts.push([i as f64, j as f64, k as f64]);
                }
            }
        }
        ExplicitStructuredGrid::from_points(dims, pts).unwrap()
    }

    #[test]
    fn basic_grid() {
        let g = make_grid_2x2x2();
        assert_eq!(g.num_points(), 8);
        assert_eq!(g.num_cells(), 1);
        assert_eq!(g.num_visible_cells(), 1);
    }

    #[test]
    fn blanking() {
        let mut g = ExplicitStructuredGrid::new([3, 3, 3]);
        assert_eq!(g.num_cells(), 8);
        assert_eq!(g.num_visible_cells(), 8);
        g.blank_cell(0, 0, 0);
        assert_eq!(g.num_visible_cells(), 7);
        assert!(!g.is_cell_visible(0, 0, 0));
        assert!(g.is_cell_visible(1, 0, 0));
        g.unblank_cell(0, 0, 0);
        assert_eq!(g.num_visible_cells(), 8);
    }

    #[test]
    fn out_of_range_blanking_does_not_alias_other_cells() {
        let mut g = ExplicitStructuredGrid::new([3, 3, 3]);
        g.blank_cell(2, 0, 0);
        assert_eq!(g.num_visible_cells(), 8);
        assert!(g.is_cell_visible(0, 1, 0));
        assert!(!g.is_cell_visible(2, 0, 0));
    }

    #[test]
    fn set_cell_visibility_rejects_wrong_length() {
        let mut g = ExplicitStructuredGrid::new([3, 3, 3]);
        assert!(g.set_cell_visibility(vec![true; 7]).is_err());
    }

    #[test]
    fn cell_point_ids() {
        let g = ExplicitStructuredGrid::new([3, 3, 2]);
        let ids = g.cell_point_ids(0, 0, 0);
        assert_eq!(ids[0], 0);
        assert_eq!(ids[1], 0);
        assert_eq!(ids, [0; 8]);
    }

    #[test]
    fn cell_point_ids_are_explicit_connectivity() {
        let mut g = ExplicitStructuredGrid::new([2, 2, 2]);
        let ids = [7, 6, 5, 4, 3, 2, 1, 0];
        g.set_cell_point_ids(0, 0, 0, ids).unwrap();
        assert_eq!(g.cell_point_ids(0, 0, 0), ids);
        assert_eq!(g.cell_points(), &[ids]);
    }

    #[test]
    fn rejects_invalid_explicit_connectivity() {
        let mut g = ExplicitStructuredGrid::new([2, 2, 2]);
        assert!(g
            .set_cell_point_ids(0, 0, 0, [0, 1, 2, 3, 4, 5, 6, 8])
            .is_err());
        assert!(g.set_cell_points(vec![[0; 8], [0; 8]]).is_err());
    }

    #[test]
    fn point_count_mismatch() {
        let pts = Points::new();
        let result = ExplicitStructuredGrid::from_points([3, 3, 3], pts);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_dimensions_rejected() {
        assert!(ExplicitStructuredGrid::from_points([3, 3, 1], Points::new()).is_err());
    }

    #[test]
    #[should_panic(expected = "cell index out of range")]
    fn cell_index_rejects_out_of_range() {
        let g = ExplicitStructuredGrid::new([3, 3, 3]);
        g.cell_index(2, 0, 0);
    }

    #[test]
    fn bounds() {
        let g = make_grid_2x2x2();
        let bb = g.bounds();
        assert_eq!(bb.x_min, 0.0);
        assert_eq!(bb.x_max, 1.0);
    }
}
