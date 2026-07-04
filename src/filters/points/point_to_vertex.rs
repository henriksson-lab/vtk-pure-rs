use crate::data::{CellArray, PolyData};

/// Convert all points in a PolyData to vertex cells.
///
/// Ensures every point has a vertex cell, useful for rendering point clouds
/// that were loaded without explicit vertex cells.
pub fn point_to_vertex(input: &PolyData) -> PolyData {
    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    *pd.point_data_mut() = input.point_data().clone();

    let n = input.points.len();
    let mut verts = CellArray::new();
    for i in 0..n {
        verts.push_cell(&[i as i64]);
    }
    pd.verts = verts;
    pd
}

/// Convert all points to a single poly-vertex cell.
pub fn point_to_poly_vertex(input: &PolyData) -> PolyData {
    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    *pd.point_data_mut() = input.point_data().clone();

    let n = input.points.len();
    if n > 0 {
        let ids: Vec<i64> = (0..n as i64).collect();
        let mut verts = CellArray::new();
        verts.push_cell(&ids);
        pd.verts = verts;
    }
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_vertex_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        assert_eq!(pd.verts.num_cells(), 0);

        let result = point_to_vertex(&pd);
        assert_eq!(result.verts.num_cells(), 3);
    }

    #[test]
    fn poly_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = point_to_poly_vertex(&pd);
        assert_eq!(result.verts.num_cells(), 1);
    }

    #[test]
    fn removes_existing_topology() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0]);

        let result = point_to_vertex(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.verts.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = point_to_vertex(&pd);
        assert_eq!(result.verts.num_cells(), 0);
    }
}
