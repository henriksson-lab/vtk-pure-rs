//! Mark boundary faces, edges, and vertices on a mesh.
//!
//! Identifies boundary entities (edges shared by only one face) and marks
//! them with point/cell data arrays. Useful for applying boundary conditions
//! or visualizing mesh boundaries.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Mark boundary vertices with a "BoundaryPoints" point data array.
///
/// A boundary vertex is any vertex that lies on a boundary edge (an edge
/// shared by only one face).
///
/// Values: 1.0 = boundary, 0.0 = interior.
pub fn mark_boundary_points(input: &PolyData) -> PolyData {
    let (is_boundary, _) = compute_boundary_marks(input);

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(
            "BoundaryPoints",
            is_boundary,
            1,
        )));
    result
}

/// Mark boundary cells with a "BoundaryCells" cell data array.
///
/// A boundary cell is any cell that has at least one boundary edge.
///
/// Values: 1.0 = boundary cell, 0.0 = interior cell.
pub fn mark_boundary_cells(input: &PolyData) -> PolyData {
    let (_, is_boundary) = compute_boundary_marks(input);

    let mut result = input.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(
            "BoundaryCells",
            is_boundary,
            1,
        )));
    result
}

/// Mark both boundary points and cells.
pub fn mark_boundary(input: &PolyData) -> PolyData {
    let (is_boundary_point, is_boundary_cell) = compute_boundary_marks(input);

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(
            "BoundaryPoints",
            is_boundary_point,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(
            "BoundaryCells",
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

fn compute_boundary_marks(input: &PolyData) -> (Vec<u8>, Vec<u8>) {
    let mut point_marks = vec![0u8; input.points.len()];
    let mut cell_marks = vec![0u8; input.total_cells()];

    mark_vertex_cells(input, &mut point_marks, &mut cell_marks);
    mark_line_cells(input, &mut point_marks, &mut cell_marks);
    mark_polygon_cells(input, &mut point_marks, &mut cell_marks);

    (point_marks, cell_marks)
}

fn mark_vertex_cells(input: &PolyData, point_marks: &mut [u8], cell_marks: &mut [u8]) {
    for (cell_id, cell) in input.verts.iter().enumerate() {
        cell_marks[cell_id] = 1;
        for &pt_id in cell {
            if let Some(mark) = point_marks.get_mut(pt_id as usize) {
                *mark = 1;
            }
        }
    }
}

fn mark_line_cells(input: &PolyData, point_marks: &mut [u8], cell_marks: &mut [u8]) {
    let offset = input.verts.num_cells();
    let mut point_use_counts = vec![0usize; input.points.len()];
    for cell in input.lines.iter() {
        for &pt_id in cell {
            if let Some(count) = point_use_counts.get_mut(pt_id as usize) {
                *count += 1;
            }
        }
    }

    for (line_id, cell) in input.lines.iter().enumerate() {
        if cell.is_empty() {
            continue;
        }
        let cell_mark = &mut cell_marks[offset + line_id];
        for &pt_id in [cell[0], cell[cell.len() - 1]].iter() {
            let uid = pt_id as usize;
            if point_use_counts.get(uid).copied().unwrap_or(0) < 2 {
                *cell_mark = 1;
                if let Some(mark) = point_marks.get_mut(uid) {
                    *mark = 1;
                }
            }
        }
    }
}

fn mark_polygon_cells(input: &PolyData, point_marks: &mut [u8], cell_marks: &mut [u8]) {
    let offset = input.verts.num_cells() + input.lines.num_cells();
    let boundary_edges = find_boundary_edges(input);
    let boundary_set: std::collections::HashSet<(usize, usize)> =
        boundary_edges.into_iter().collect();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = (a.min(b), a.max(b));
            if boundary_set.contains(&edge) {
                cell_marks[offset + poly_id] = 1;
                if let Some(mark) = point_marks.get_mut(a) {
                    *mark = 1;
                }
                if let Some(mark) = point_marks.get_mut(b) {
                    *mark = 1;
                }
            }
        }
    }
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
        let arr = result.point_data().get_array("BoundaryPoints").unwrap();
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
        let arr = result.cell_data().get_array("BoundaryCells").unwrap();
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
        assert!(result.point_data().get_array("BoundaryPoints").is_some());
        assert!(result.cell_data().get_array("BoundaryCells").is_some());
    }

    #[test]
    fn line_boundary_uses_all_point_links() {
        let mut mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        mesh.lines.push_cell(&[0, 1, 2]);
        mesh.lines.push_cell(&[3, 1]);

        let result = mark_boundary(&mesh);
        let points = result.point_data().get_array("BoundaryPoints").unwrap();
        let mut buf = [0.0f64];

        points.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
        points.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        points.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
        points.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0);
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
