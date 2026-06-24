use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extract boundary edges from a polygon mesh.
///
/// A boundary edge is one that belongs to exactly one polygon face.
/// Returns a new PolyData containing the boundary edges as lines,
/// with the same point coordinates as the input.
pub fn extract_boundary_edges(input: &PolyData) -> PolyData {
    // Count how many faces share each edge.
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 2 {
            continue;
        }
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(edge).or_insert(0usize) += 1;
        }
    }

    let mut lines = CellArray::new();
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            lines.push_cell(&[a as i64, b as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.lines = lines;
    pd
}

/// Count the number of boundary edges in the mesh.
pub fn count_boundary_edges(input: &PolyData) -> usize {
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 2 {
            continue;
        }
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(edge).or_insert(0usize) += 1;
        }
    }

    edge_count.values().filter(|&&c| c == 1).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_all_boundary() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let boundary = extract_boundary_edges(&pd);
        // A single triangle has 3 boundary edges
        assert_eq!(boundary.lines.num_cells(), 3);
        assert_eq!(count_boundary_edges(&pd), 3);
    }

    #[test]
    fn two_triangles_shared_edge() {
        // Triangles share edge 0-1; remaining 4 edges are boundary
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        assert_eq!(count_boundary_edges(&pd), 4);
        let boundary = extract_boundary_edges(&pd);
        assert_eq!(boundary.lines.num_cells(), 4);
    }

    #[test]
    fn closed_tetrahedron_no_boundary() {
        // A tetrahedron (4 triangular faces) has no boundary edges —
        // every edge is shared by exactly 2 faces.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        assert_eq!(count_boundary_edges(&pd), 0);
        let boundary = extract_boundary_edges(&pd);
        assert_eq!(boundary.lines.num_cells(), 0);
    }
}
