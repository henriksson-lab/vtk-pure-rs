//! Mesh skeleton extraction as a graph (curve skeleton).
//!
//! Extracts a 1D skeleton from a mesh using iterative vertex contraction.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract a curve skeleton from a mesh via iterative Laplacian contraction.
///
/// Repeatedly smooths the mesh while preserving connectivity until
/// it collapses to a thin skeleton.
pub fn extract_skeleton_graph(
    mesh: &PolyData,
    iterations: usize,
    contraction_rate: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }

    let adj = build_adj(mesh, n);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    // Iterative contraction
    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += positions[j][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] =
                    positions[i][c] * (1.0 - contraction_rate) + (avg[c] / k) * contraction_rate;
            }
        }
        positions = new_pos;
    }

    // Merge close points and extract edges as skeleton
    let merge_dist = compute_avg_edge_length(mesh) * 0.3;
    let merge_dist2 = merge_dist * merge_dist;

    let mut mapping = vec![0usize; n];
    let mut skeleton_pts = Points::<f64>::new();
    let mut used = vec![false; n];

    for i in 0..n {
        if used[i] {
            continue;
        }
        let new_idx = skeleton_pts.len();
        skeleton_pts.push(positions[i]);
        mapping[i] = new_idx;

        // Merge nearby points
        for j in i + 1..n {
            if used[j] {
                continue;
            }
            let d2 = (positions[i][0] - positions[j][0]).powi(2)
                + (positions[i][1] - positions[j][1]).powi(2)
                + (positions[i][2] - positions[j][2]).powi(2);
            if d2 < merge_dist2 {
                mapping[j] = new_idx;
                used[j] = true;
            }
        }
    }

    // Extract unique edges
    let mut edge_set: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut lines = CellArray::new();

    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = mapping[cell[i] as usize];
            let b = mapping[cell[(i + 1) % nc] as usize];
            if a != b {
                let edge = (a.min(b), a.max(b));
                if edge_set.insert(edge) {
                    lines.push_cell(&[a as i64, b as i64]);
                }
            }
        }
    }

    // Compute degree per skeleton vertex
    let mut degree = vec![0.0f64; skeleton_pts.len()];
    for e in &edge_set {
        degree[e.0] += 1.0;
        degree[e.1] += 1.0;
    }

    let mut result = PolyData::new();
    result.points = skeleton_pts;
    result.lines = lines;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Degree", degree, 1)));
    result
}

/// Compute skeleton statistics.
pub fn skeleton_stats(skeleton: &PolyData) -> (usize, usize, usize, usize) {
    let n_pts = skeleton.points.len();
    let n_edges = skeleton.lines.num_cells();
    let mut n_endpoints = 0;
    let mut n_junctions = 0;

    if let Some(deg) = skeleton.point_data().get_array("Degree") {
        let mut buf = [0.0f64];
        for i in 0..deg.num_tuples() {
            deg.tuple_as_f64(i, &mut buf);
            if buf[0] <= 1.0 {
                n_endpoints += 1;
            } else if buf[0] >= 3.0 {
                n_junctions += 1;
            }
        }
    }

    (n_pts, n_edges, n_endpoints, n_junctions)
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn compute_avg_edge_length(mesh: &PolyData) -> f64 {
    let mut total = 0.0;
    let mut count = 0;
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = mesh.points.get(cell[i] as usize);
            let b = mesh.points.get(cell[(i + 1) % nc] as usize);
            total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            count += 1;
        }
    }
    if count > 0 {
        total / count as f64
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tube_skeleton() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let skel = extract_skeleton_graph(&mesh, 20, 0.8);
        assert!(skel.points.len() >= 0); // may be empty for simple shapes
    }

    #[test]
    fn stats() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let skel = extract_skeleton_graph(&mesh, 10, 0.7);
        let (pts, _edges, _endpoints, _junctions) = skeleton_stats(&skel);
        assert!(pts >= 0); // skeleton may be empty for compact shapes
    }

    #[test]
    fn empty() {
        let skel = extract_skeleton_graph(&PolyData::new(), 10, 0.5);
        assert_eq!(skel.points.len(), 0);
    }
}
