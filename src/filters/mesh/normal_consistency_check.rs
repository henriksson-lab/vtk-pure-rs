use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Result of a normal consistency check.
pub struct ConsistencyResult {
    /// True if all face normals are consistently oriented.
    pub is_consistent: bool,
    /// Number of edge pairs where adjacent faces have inconsistent orientation.
    pub inconsistent_count: usize,
}

/// Check whether all face normals in a triangle mesh are consistently oriented.
///
/// Two adjacent triangles sharing an edge (u, v) are consistent if one triangle
/// traverses the edge as (u, v) and the other as (v, u). If both traverse it
/// in the same direction, that edge pair is inconsistent.
///
/// Returns a `ConsistencyResult` with overall consistency and the count of
/// inconsistent edge pairs.
pub fn check_normal_consistency(input: &PolyData) -> ConsistencyResult {
    // Map directed edge (u,v) -> list of cell indices that use that directed edge
    let mut edge_map: HashMap<(i64, i64), Vec<usize>> = HashMap::new();

    for (ci, cell) in input.polys.iter().enumerate() {
        let n: usize = cell.len();
        if n < 2 {
            continue;
        }
        for i in 0..n {
            let u: i64 = cell[i];
            let v: i64 = cell[(i + 1) % n];
            edge_map.entry((u, v)).or_default().push(ci);
        }
    }

    let mut inconsistent: usize = 0;
    let mut visited: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();

    for &(u, v) in edge_map.keys() {
        let key = if u < v { (u, v) } else { (v, u) };
        if !visited.insert(key) {
            continue;
        }
        let fwd: usize = edge_map.get(&(u, v)).map_or(0, |v| v.len());
        let rev: usize = edge_map.get(&(v, u)).map_or(0, |v| v.len());

        inconsistent += fwd.saturating_sub(1) * fwd / 2;
        inconsistent += rev.saturating_sub(1) * rev / 2;
    }

    ConsistencyResult {
        is_consistent: inconsistent == 0,
        inconsistent_count: inconsistent,
    }
}

/// Add an "IsConsistent" cell data array.
///
/// Each cell gets 1.0 if none of its edges are inconsistent with neighbors,
/// or 0.0 if at least one edge is inconsistent.
pub fn add_consistency_cell_data(input: &PolyData) -> PolyData {
    // Map directed edge (u,v) -> list of cell indices
    let mut edge_map: HashMap<(i64, i64), Vec<usize>> = HashMap::new();

    for (ci, cell) in input.polys.iter().enumerate() {
        let n: usize = cell.len();
        if n < 2 {
            continue;
        }
        for i in 0..n {
            let u: i64 = cell[i];
            let v: i64 = cell[(i + 1) % n];
            edge_map.entry((u, v)).or_default().push(ci);
        }
    }

    let num_cells: usize = input.polys.num_cells();
    let mut consistent: Vec<f64> = vec![1.0; num_cells];

    let mut visited: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();
    for &(u, v) in edge_map.keys() {
        let key = if u < v { (u, v) } else { (v, u) };
        if !visited.insert(key) {
            continue;
        }
        for cells in [edge_map.get(&(u, v)), edge_map.get(&(v, u))]
            .into_iter()
            .flatten()
        {
            if cells.len() >= 2 {
                for &ci in cells {
                    consistent[ci] = 0.0;
                }
            }
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IsConsistent",
            consistent,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_pair() {
        // Two triangles sharing edge (1,2) with consistent winding
        // Triangle 0: 0->1->2, Triangle 1: 1->3->2
        // Edge (1,2) in tri0, edge (2,1) in tri1 (via 3->2...1) - wait let me be explicit
        // Tri0: edges (0,1), (1,2), (2,0)
        // Tri1: edges (1,3), (3,2), (2,1)
        // Shared edge: tri0 has (1,2), tri1 has (2,1) -> consistent
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = check_normal_consistency(&pd);
        assert!(result.is_consistent);
        assert_eq!(result.inconsistent_count, 0);
    }

    #[test]
    fn inconsistent_pair() {
        // Two triangles sharing edge (1,2) both in the same direction
        // Tri0: 0->1->2, Tri1: 2->1->3
        // Tri0 edges: (0,1),(1,2),(2,0)
        // Tri1 edges: (2,1),(1,3),(3,2) -- edge (2,1) vs tri0's (1,2) -> ok wait
        // Let me flip: Tri1: 1->2->3
        // Tri1 edges: (1,2),(2,3),(3,1) -- both have (1,2) -> inconsistent
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 2, 3]],
        );
        let result = check_normal_consistency(&pd);
        assert!(!result.is_consistent);
        assert!(result.inconsistent_count > 0);
    }

    #[test]
    fn cell_data_marks_inconsistent() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 2, 3]],
        );
        let result = add_consistency_cell_data(&pd);
        let arr = result.cell_data().get_array("IsConsistent").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        // Both faces share the inconsistent edge, so both should be 0.0
        let mut v0 = [0.0f64];
        let mut v1 = [0.0f64];
        arr.tuple_as_f64(0, &mut v0);
        arr.tuple_as_f64(1, &mut v1);
        assert!((v0[0] - 0.0).abs() < 1e-10);
        assert!((v1[0] - 0.0).abs() < 1e-10);
    }
}
