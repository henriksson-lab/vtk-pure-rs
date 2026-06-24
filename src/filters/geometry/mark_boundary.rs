//! Mark boundary faces, edges, and vertices on a mesh.
//!
//! Identifies boundary entities (edges shared by only one face) and marks
//! them with point/cell data arrays. Useful for applying boundary conditions
//! or visualizing mesh boundaries.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Mark boundary vertices with a "BoundaryPoint" point data array.
///
/// A boundary vertex is any vertex that lies on a boundary edge (an edge
/// shared by only one face).
///
/// Values: 1.0 = boundary, 0.0 = interior.
pub fn mark_boundary_points(input: &PolyData) -> PolyData {
    let n_pts = input.points.len();
    let mut is_boundary = vec![0.0f64; n_pts];

    let boundary_edges = find_boundary_edges(input);
    for (a, b) in &boundary_edges {
        is_boundary[*a] = 1.0;
        is_boundary[*b] = 1.0;
    }

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryPoint",
            is_boundary,
            1,
        )));
    result
}

/// Mark boundary cells with a "BoundaryCell" cell data array.
///
/// A boundary cell is any cell that has at least one boundary edge.
///
/// Values: 1.0 = boundary cell, 0.0 = interior cell.
pub fn mark_boundary_cells(input: &PolyData) -> PolyData {
    let boundary_edges = find_boundary_edges(input);
    let boundary_set: std::collections::HashSet<(usize, usize)> =
        boundary_edges.into_iter().collect();

    let n_cells = input.polys.num_cells();
    let mut is_boundary = vec![0.0f64; n_cells];

    for (ci, cell) in input.polys.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = (a.min(b), a.max(b));
            if boundary_set.contains(&edge) {
                is_boundary[ci] = 1.0;
                break;
            }
        }
    }

    let mut result = input.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryCell",
            is_boundary,
            1,
        )));
    result
}

/// Mark both boundary points and cells.
pub fn mark_boundary(input: &PolyData) -> PolyData {
    let boundary_edges = find_boundary_edges(input);
    let boundary_set: std::collections::HashSet<(usize, usize)> =
        boundary_edges.iter().cloned().collect();

    let n_pts = input.points.len();
    let n_cells = input.polys.num_cells();

    let mut is_boundary_point = vec![0.0f64; n_pts];
    let mut is_boundary_cell = vec![0.0f64; n_cells];

    for &(a, b) in &boundary_edges {
        is_boundary_point[a] = 1.0;
        is_boundary_point[b] = 1.0;
    }

    for (ci, cell) in input.polys.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = (a.min(b), a.max(b));
            if boundary_set.contains(&edge) {
                is_boundary_cell[ci] = 1.0;
                break;
            }
        }
    }

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryPoint",
            is_boundary_point,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryCell",
            is_boundary_cell,
            1,
        )));
    result
}

/// Count boundary edges.
pub fn count_boundary_edges(input: &PolyData) -> usize {
    find_boundary_edges(input).len()
}

/// Count boundary vertices.
pub fn count_boundary_vertices(input: &PolyData) -> usize {
    let edges = find_boundary_edges(input);
    let mut verts: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (a, b) in &edges {
        verts.insert(*a);
        verts.insert(*b);
    }
    verts.len()
}

fn find_boundary_edges(input: &PolyData) -> Vec<(usize, usize)> {
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = (a.min(b), a.max(b));
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    }

    edge_count
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(edge, _)| edge)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_all_boundary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(count_boundary_edges(&mesh), 3);
        assert_eq!(count_boundary_vertices(&mesh), 3);
    }

    #[test]
    fn two_triangles_shared_edge() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        // Edge 0-1 is shared, so 4 boundary edges remain
        assert_eq!(count_boundary_edges(&mesh), 4);
    }

    #[test]
    fn mark_boundary_points_test() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = mark_boundary_points(&mesh);
        let arr = result.point_data().get_array("BoundaryPoint").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0); // all boundary
        }
    }

    #[test]
    fn mark_boundary_cells_test() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = mark_boundary_cells(&mesh);
        let arr = result.cell_data().get_array("BoundaryCell").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn mark_both() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = mark_boundary(&mesh);
        assert!(result.point_data().get_array("BoundaryPoint").is_some());
        assert!(result.cell_data().get_array("BoundaryCell").is_some());
    }

    #[test]
    fn closed_mesh_no_boundary() {
        // Tetrahedron surface (closed)
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        // Each edge is shared by 2 faces
        assert_eq!(count_boundary_edges(&mesh), 0);
    }
}
