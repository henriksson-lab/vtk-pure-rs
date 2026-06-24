use crate::types::BoundingBox;

use crate::data::traits::{DataObject, DataSet};
use crate::data::{CellArray, DataSetAttributes, FieldData, Points};

/// Polygonal mesh consisting of vertices, lines, polygons, and triangle strips.
///
/// Analogous to VTK's `vtkPolyData`. Points are stored explicitly, with four
/// separate `CellArray`s for the four cell categories.
///
/// # Examples
///
/// ```
/// use vtk_pure_rs::data::PolyData;
///
/// // Create a triangle mesh
/// let pd = PolyData::from_triangles(
///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
///     vec![[0, 1, 2]],
/// );
/// assert_eq!(pd.points.len(), 3);
/// assert_eq!(pd.polys.num_cells(), 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PolyData {
    pub points: Points<f64>,
    pub verts: CellArray,
    pub lines: CellArray,
    pub polys: CellArray,
    pub strips: CellArray,
    point_data: DataSetAttributes,
    cell_data: DataSetAttributes,
    field_data: FieldData,
}

impl PolyData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a PolyData from points and triangle connectivity.
    ///
    /// Each element of `triangles` is `[i0, i1, i2]` indexing into `points`.
    pub fn from_triangles(points: Vec<[f64; 3]>, triangles: Vec<[i64; 3]>) -> Self {
        let pts = Points::from_vec(points);
        let mut polys = CellArray::new();
        for tri in &triangles {
            polys.push_cell(&[tri[0], tri[1], tri[2]]);
        }
        Self {
            points: pts,
            polys,
            ..Default::default()
        }
    }

    /// Create a PolyData from points and generic polygon cells.
    ///
    /// Each element of `cells` is a Vec of point indices for one polygon.
    /// Supports mixed triangle/quad/polygon meshes.
    pub fn from_polygons(points: Vec<[f64; 3]>, cells: Vec<Vec<i64>>) -> Self {
        let pts = Points::from_vec(points);
        let mut polys = CellArray::new();
        for cell in &cells {
            polys.push_cell(cell);
        }
        Self {
            points: pts,
            polys,
            ..Default::default()
        }
    }

    /// Create a PolyData from points and quad connectivity.
    ///
    /// Each element of `quads` is `[i0, i1, i2, i3]` indexing into `points`.
    pub fn from_quads(points: Vec<[f64; 3]>, quads: Vec<[i64; 4]>) -> Self {
        let pts = Points::from_vec(points);
        let mut polys = CellArray::new();
        for q in &quads {
            polys.push_cell(&[q[0], q[1], q[2], q[3]]);
        }
        Self {
            points: pts,
            polys,
            ..Default::default()
        }
    }

    /// Create a PolyData with line cells.
    ///
    /// Each element of `segments` is `[i0, i1]` indexing into `points`.
    pub fn from_lines(points: Vec<[f64; 3]>, segments: Vec<[i64; 2]>) -> Self {
        let pts = Points::from_vec(points);
        let mut lines = CellArray::new();
        for seg in &segments {
            lines.push_cell(&[seg[0], seg[1]]);
        }
        Self {
            points: pts,
            lines,
            ..Default::default()
        }
    }

    /// Create a PolyData from flat coordinate and index arrays.
    ///
    /// `coords` is `[x0,y0,z0, x1,y1,z1, ...]` (length must be divisible by 3).
    /// `indices` is `[i0,i1,i2, i3,i4,i5, ...]` (length must be divisible by 3, triangles).
    ///
    /// This is the most common format when receiving data from other libraries,
    /// GPU buffers, or numpy arrays.
    ///
    /// ```
    /// use vtk_pure_rs::data::PolyData;
    ///
    /// let pd = PolyData::from_flat_arrays(
    ///     &[0.0,0.0,0.0, 1.0,0.0,0.0, 0.0,1.0,0.0],
    ///     &[0, 1, 2],
    /// );
    /// assert_eq!(pd.points.len(), 3);
    /// assert_eq!(pd.polys.num_cells(), 1);
    /// ```
    pub fn from_flat_arrays(coords: &[f64], indices: &[i64]) -> Self {
        assert!(
            coords.len() % 3 == 0,
            "coords length must be divisible by 3"
        );
        assert!(
            indices.len() % 3 == 0,
            "indices length must be divisible by 3"
        );

        let pts: Vec<[f64; 3]> = coords.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
        let tris: Vec<[i64; 3]> = indices
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        Self::from_triangles(pts, tris)
    }

    /// Create a PolyData with points only (no cells).
    ///
    /// Useful for point clouds that will later have cells added.
    pub fn from_points(points: Vec<[f64; 3]>) -> Self {
        Self {
            points: Points::from_vec(points),
            ..Default::default()
        }
    }

    /// Create a PolyData with vertex cells (one vertex per point).
    pub fn from_vertices(points: Vec<[f64; 3]>) -> Self {
        let n = points.len();
        let pts = Points::from_vec(points);
        let mut verts = CellArray::new();
        for i in 0..n {
            verts.push_cell(&[i as i64]);
        }
        Self {
            points: pts,
            verts,
            ..Default::default()
        }
    }

    /// Create a PolyData with a single polyline through all points.
    pub fn from_polyline(points: Vec<[f64; 3]>) -> Self {
        let n = points.len();
        let pts = Points::from_vec(points);
        let mut lines = CellArray::new();
        let ids: Vec<i64> = (0..n as i64).collect();
        lines.push_cell(&ids);
        Self {
            points: pts,
            lines,
            ..Default::default()
        }
    }

    /// Push a single point and return its index.
    pub fn push_point(&mut self, point: [f64; 3]) -> i64 {
        let idx = self.points.len() as i64;
        self.points.push(point);
        idx
    }

    /// Push a triangle cell from 3 point indices.
    pub fn push_triangle(&mut self, i0: i64, i1: i64, i2: i64) {
        self.polys.push_cell(&[i0, i1, i2]);
    }

    /// Push a quad cell from 4 point indices.
    pub fn push_quad(&mut self, i0: i64, i1: i64, i2: i64, i3: i64) {
        self.polys.push_cell(&[i0, i1, i2, i3]);
    }

    /// Push a line cell from 2 point indices.
    pub fn push_line(&mut self, i0: i64, i1: i64) {
        self.lines.push_cell(&[i0, i1]);
    }

    /// Total number of cells across all four categories.
    pub fn total_cells(&self) -> usize {
        self.verts.num_cells()
            + self.lines.num_cells()
            + self.polys.num_cells()
            + self.strips.num_cells()
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

    /// Access field data directly (convenience, same as DataObject::field_data).
    pub fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    /// Mutable access to field data.
    pub fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }

    /// Reverse the winding order of all polygon cells.
    ///
    /// Useful for flipping normals direction.
    pub fn reverse_cells(&mut self) {
        let mut new_polys = CellArray::new();
        for cell in self.polys.iter() {
            let reversed: Vec<i64> = cell.iter().copied().rev().collect();
            new_polys.push_cell(&reversed);
        }
        self.polys = new_polys;
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

    /// Builder: set active scalars by name.
    pub fn with_active_scalars(mut self, name: &str) -> Self {
        self.point_data.set_active_scalars(name);
        self
    }

    /// Create from separate X, Y, Z coordinate arrays and triangle indices.
    pub fn from_xyz_arrays(x: &[f64], y: &[f64], z: &[f64], triangles: &[[i64; 3]]) -> Self {
        assert_eq!(x.len(), y.len());
        assert_eq!(x.len(), z.len());
        let pts: Vec<[f64; 3]> = x
            .iter()
            .zip(y.iter())
            .zip(z.iter())
            .map(|((&xi, &yi), &zi)| [xi, yi, zi])
            .collect();
        Self::from_triangles(pts, triangles.to_vec())
    }

    /// Append another PolyData in-place (mutates self).
    pub fn append(&mut self, other: &PolyData) {
        let base = self.points.len() as i64;
        for p in &other.points {
            self.points.push(p);
        }
        for cell in other.polys.iter() {
            let offset: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            self.polys.push_cell(&offset);
        }
        for cell in other.lines.iter() {
            let offset: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            self.lines.push_cell(&offset);
        }
        for cell in other.verts.iter() {
            let offset: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            self.verts.push_cell(&offset);
        }
        for cell in other.strips.iter() {
            let offset: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            self.strips.push_cell(&offset);
        }
    }

    /// Number of polygon faces.
    pub fn num_polys(&self) -> usize {
        self.polys.num_cells()
    }

    /// Count actual triangles (cells with exactly 3 points).
    pub fn num_triangles(&self) -> usize {
        self.polys.iter().filter(|c| c.len() == 3).count()
    }

    /// Compute the centroid (average position of all points).
    pub fn centroid(&self) -> [f64; 3] {
        self.points.centroid()
    }

    /// Compute a bounding sphere: (center, radius).
    pub fn bounding_sphere(&self) -> ([f64; 3], f64) {
        let center = self.centroid();
        let mut max_r2 = 0.0f64;
        for p in &self.points {
            let dx = p[0] - center[0];
            let dy = p[1] - center[1];
            let dz = p[2] - center[2];
            max_r2 = max_r2.max(dx * dx + dy * dy + dz * dz);
        }
        (center, max_r2.sqrt())
    }

    /// Whether all polygon cells are triangles.
    pub fn is_all_triangles(&self) -> bool {
        self.polys.num_cells() > 0 && self.polys.iter().all(|c| c.len() == 3)
    }

    /// Number of line cells.
    pub fn num_lines(&self) -> usize {
        self.lines.num_cells()
    }

    /// Number of vertex cells.
    pub fn num_verts(&self) -> usize {
        self.verts.num_cells()
    }

    /// Count unique edges in the polygon mesh.
    pub fn num_edges(&self) -> usize {
        let mut edges = std::collections::HashSet::new();
        for cell in self.polys.iter() {
            let n = cell.len();
            for i in 0..n {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % n] as usize;
                edges.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        edges.len()
    }

    /// Add a scalar (1-component f64) point data array and set it as active scalars.
    ///
    /// ```
    /// use vtk_pure_rs::data::PolyData;
    ///
    /// let mut pd = PolyData::from_triangles(
    ///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    ///     vec![[0, 1, 2]],
    /// );
    /// pd.add_scalars("temperature", vec![10.0, 20.0, 30.0]);
    /// assert_eq!(pd.get_scalars("temperature"), Some(vec![10.0, 20.0, 30.0]));
    /// ```
    pub fn add_scalars(&mut self, name: &str, values: Vec<f64>) {
        let arr = crate::data::DataArray::from_vec(name, values, 1);
        self.point_data
            .add_array(crate::data::AnyDataArray::F64(arr));
        self.point_data.set_active_scalars(name);
    }

    /// Add a vector (3-component f64) point data array.
    pub fn add_vectors(&mut self, name: &str, values: Vec<[f64; 3]>) {
        let flat: Vec<f64> = values.into_iter().flat_map(|v| v).collect();
        let arr = crate::data::DataArray::from_vec(name, flat, 3);
        self.point_data
            .add_array(crate::data::AnyDataArray::F64(arr));
        self.point_data.set_active_vectors(name);
    }

    /// Get scalar values by name as `Vec<f64>`. Returns None if array not found.
    pub fn get_scalars(&self, name: &str) -> Option<Vec<f64>> {
        self.point_data.get_array(name).map(|a| a.to_f64_vec())
    }

    /// Get vector values by name. Returns None if array not found.
    pub fn get_vectors(&self, name: &str) -> Option<Vec<[f64; 3]>> {
        let arr = self.point_data.get_array(name)?;
        if arr.num_components() != 3 {
            return None;
        }
        let mut result = Vec::with_capacity(arr.num_tuples());
        let mut buf = [0.0f64; 3];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            result.push(buf);
        }
        Some(result)
    }

    /// Human-readable summary of the PolyData.
    pub fn summary(&self) -> String {
        format!(
            "PolyData: {} points, {} polys, {} lines, {} verts, {} point arrays, {} cell arrays",
            self.points.len(),
            self.polys.num_cells(),
            self.lines.num_cells(),
            self.verts.num_cells(),
            self.point_data.num_arrays(),
            self.cell_data.num_arrays(),
        )
    }

    /// Convert point coordinates and point data to a Table.
    ///
    /// Creates columns "x", "y", "z" from point coordinates,
    /// plus one column per point data array.
    pub fn to_table(&self) -> crate::data::Table {
        let n = self.points.len();
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        let mut z = Vec::with_capacity(n);
        for p in &self.points {
            x.push(p[0]);
            y.push(p[1]);
            z.push(p[2]);
        }
        let mut table = crate::data::Table::new();
        table.add_column(crate::data::AnyDataArray::F64(
            crate::data::DataArray::from_vec("x", x, 1),
        ));
        table.add_column(crate::data::AnyDataArray::F64(
            crate::data::DataArray::from_vec("y", y, 1),
        ));
        table.add_column(crate::data::AnyDataArray::F64(
            crate::data::DataArray::from_vec("z", z, 1),
        ));

        for i in 0..self.point_data.num_arrays() {
            if let Some(arr) = self.point_data.get_array_by_index(i) {
                table.add_column(arr.clone());
            }
        }
        table
    }

    /// Check approximate equality with another PolyData (for testing).
    pub fn approx_eq(&self, other: &PolyData, tolerance: f64) -> bool {
        if self.points.len() != other.points.len() {
            return false;
        }
        if self.polys.num_cells() != other.polys.num_cells() {
            return false;
        }
        for i in 0..self.points.len() {
            let a = self.points.get(i);
            let b = other.points.get(i);
            if (a[0] - b[0]).abs() > tolerance
                || (a[1] - b[1]).abs() > tolerance
                || (a[2] - b[2]).abs() > tolerance
            {
                return false;
            }
        }
        for (ca, cb) in self.polys.iter().zip(other.polys.iter()) {
            if ca != cb {
                return false;
            }
        }
        true
    }
}

impl DataObject for PolyData {
    fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

impl DataSet for PolyData {
    fn num_points(&self) -> usize {
        self.points.len()
    }

    fn num_cells(&self) -> usize {
        self.total_cells()
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

impl std::fmt::Display for PolyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_triangles() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(pd.num_points(), 3);
        assert_eq!(pd.num_cells(), 1);
        assert_eq!(pd.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn bounds() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [0.5, 1.0, 1.5]],
            vec![[0, 1, 2]],
        );
        let bb = pd.bounds();
        assert_eq!(bb.x_min, 0.0);
        assert_eq!(bb.x_max, 1.0);
        assert_eq!(bb.y_max, 2.0);
        assert_eq!(bb.z_max, 3.0);
    }

    #[test]
    fn empty_poly_data() {
        let pd = PolyData::new();
        assert_eq!(pd.num_points(), 0);
        assert_eq!(pd.num_cells(), 0);
    }

    #[test]
    fn from_quads() {
        let pd = PolyData::from_quads(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        assert_eq!(pd.num_points(), 4);
        assert_eq!(pd.polys.num_cells(), 1);
        assert_eq!(pd.polys.cell(0).len(), 4);
    }

    #[test]
    fn from_lines() {
        let pd = PolyData::from_lines(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
            vec![[0, 1], [1, 2]],
        );
        assert_eq!(pd.num_points(), 3);
        assert_eq!(pd.lines.num_cells(), 2);
    }

    #[test]
    fn from_vertices() {
        let pd = PolyData::from_vertices(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        assert_eq!(pd.num_points(), 2);
        assert_eq!(pd.verts.num_cells(), 2);
    }

    #[test]
    fn from_polyline() {
        let pd = PolyData::from_polyline(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]]);
        assert_eq!(pd.num_points(), 3);
        assert_eq!(pd.lines.num_cells(), 1);
        assert_eq!(pd.lines.cell(0).len(), 3);
    }

    #[test]
    fn from_xyz_arrays() {
        let x = vec![0.0, 1.0, 0.0];
        let y = vec![0.0, 0.0, 1.0];
        let z = vec![0.0, 0.0, 0.0];
        let pd = PolyData::from_xyz_arrays(&x, &y, &z, &[[0, 1, 2]]);
        assert_eq!(pd.num_points(), 3);
        assert_eq!(pd.num_polys(), 1);
    }

    #[test]
    fn append() {
        let mut a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        a.append(&b);
        assert_eq!(a.points.len(), 6);
        assert_eq!(a.polys.num_cells(), 2);
        assert_eq!(a.polys.cell(1), &[3, 4, 5]);
    }

    #[test]
    fn edge_count() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(pd.num_edges(), 3);
    }

    #[test]
    fn summary() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = pd.summary();
        assert!(s.contains("3 points"));
        assert!(s.contains("1 polys"));
    }

    #[test]
    fn approx_eq() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = a.clone();
        assert!(a.approx_eq(&b, 1e-10));
    }

    #[test]
    fn num_convenience() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(pd.num_polys(), 1);
        assert_eq!(pd.num_lines(), 0);
        assert_eq!(pd.num_verts(), 0);
    }

    #[test]
    fn display() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = format!("{pd}");
        assert!(s.contains("3 points"));
    }

    #[test]
    fn to_table() {
        let mut pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let s = crate::data::DataArray::from_vec("temp", vec![10.0f64, 20.0, 30.0], 1);
        pd.point_data_mut()
            .add_array(crate::data::AnyDataArray::F64(s));

        let table = pd.to_table();
        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.num_columns(), 4); // x, y, z, temp
        assert_eq!(table.value_f64(0, "x"), Some(1.0));
        assert_eq!(table.value_f64(1, "y"), Some(5.0));
        assert_eq!(table.value_f64(2, "temp"), Some(30.0));
    }

    #[test]
    fn from_points_no_cells() {
        let pd = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert_eq!(pd.points.len(), 2);
        assert_eq!(pd.polys.num_cells(), 0);
        assert_eq!(pd.verts.num_cells(), 0);
    }

    #[test]
    fn from_polygons_mixed() {
        let pd = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![vec![0, 1, 2], vec![0, 2, 3], vec![1, 4, 2]],
        );
        assert_eq!(pd.points.len(), 5);
        assert_eq!(pd.polys.num_cells(), 3);
    }

    #[test]
    fn triangle_counting() {
        let pd = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2], vec![0, 1, 2, 3]],
        );
        assert_eq!(pd.num_triangles(), 1);
        assert!(!pd.is_all_triangles());

        let tri = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(tri.is_all_triangles());
    }

    #[test]
    fn add_get_scalars() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.add_scalars("temp", vec![10.0, 20.0, 30.0]);
        let vals = pd.get_scalars("temp").unwrap();
        assert_eq!(vals, vec![10.0, 20.0, 30.0]);
        assert!(pd.get_scalars("nonexistent").is_none());
    }

    #[test]
    fn add_get_vectors() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.add_vectors(
            "velocity",
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        );
        let vecs = pd.get_vectors("velocity").unwrap();
        assert_eq!(vecs.len(), 3);
        assert_eq!(vecs[0], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn incremental_building() {
        let mut pd = PolyData::new();
        let i0 = pd.push_point([0.0, 0.0, 0.0]);
        let i1 = pd.push_point([1.0, 0.0, 0.0]);
        let i2 = pd.push_point([0.0, 1.0, 0.0]);
        let i3 = pd.push_point([1.0, 1.0, 0.0]);
        pd.push_triangle(i0, i1, i2);
        pd.push_triangle(i1, i3, i2);
        pd.push_line(i0, i1);

        assert_eq!(pd.points.len(), 4);
        assert_eq!(pd.num_polys(), 2);
        assert_eq!(pd.num_lines(), 1);
        assert_eq!(pd.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn reverse_cells() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.reverse_cells();
        assert_eq!(pd.polys.cell(0), &[2, 1, 0]);
    }

    #[test]
    fn field_data_access() {
        let mut pd = PolyData::new();
        assert!(pd.field_data().is_empty());
        pd.field_data_mut()
            .add_array(crate::data::AnyDataArray::F64(
                crate::data::DataArray::from_vec("meta", vec![42.0], 1),
            ));
        assert!(pd.field_data().has_array("meta"));
    }
}
