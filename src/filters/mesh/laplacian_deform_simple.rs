use std::collections::{HashMap, HashSet};

use crate::data::PolyData;

/// Simple Laplacian surface deformation.
///
/// Given anchor vertices (fixed in place at target positions) and the rest of the mesh,
/// iteratively solve for vertex positions that preserve the original Laplacian coordinates
/// while respecting the anchor constraints.
///
/// `anchors` is a slice of `(vertex_index, target_position)` pairs. Each anchor vertex
/// will be moved to its target position, and the remaining vertices will be adjusted
/// to preserve the differential coordinates (Laplacian) of the original mesh.
///
/// `iterations` controls the number of iterative relaxation passes.
pub fn laplacian_deform_simple(
    input: &PolyData,
    anchors: &[(usize, [f64; 3])],
    iterations: usize,
) -> PolyData {
    let mut output = input.clone();
    let n: usize = output.points.len();
    if n == 0 || iterations == 0 || anchors.is_empty() {
        return output;
    }

    // Build adjacency from polygon connectivity
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        let len: usize = cell.len();
        for j in 0..len {
            let a: usize = cell[j] as usize;
            let b: usize = cell[(j + 1) % len] as usize;
            if a < n && b < n {
                neighbors[a].insert(b);
                neighbors[b].insert(a);
            }
        }
    }

    // Compute original Laplacian coordinates: L(v) = v - average(neighbors)
    let mut laplacian: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];
    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        let nbrs = &neighbors[i];
        if nbrs.is_empty() {
            continue;
        }
        let count: f64 = nbrs.len() as f64;
        let mut avg: [f64; 3] = [0.0, 0.0, 0.0];
        for &nb in nbrs {
            let q: [f64; 3] = input.points.get(nb);
            avg[0] += q[0];
            avg[1] += q[1];
            avg[2] += q[2];
        }
        avg[0] /= count;
        avg[1] /= count;
        avg[2] /= count;
        laplacian[i] = [p[0] - avg[0], p[1] - avg[1], p[2] - avg[2]];
    }

    // Build anchor map for fast lookup
    let anchor_map: HashMap<usize, [f64; 3]> = anchors.iter().cloned().collect();

    // Set anchor vertices to their target positions initially
    for (&idx, &target) in &anchor_map {
        if idx < n {
            output.points.set(idx, target);
        }
    }

    // Iterative relaxation: for non-anchor vertices, move toward
    // the position that preserves the Laplacian coordinates
    for _ in 0..iterations {
        let mut new_positions: Vec<[f64; 3]> = Vec::with_capacity(n);

        for i in 0..n {
            if anchor_map.contains_key(&i) {
                // Anchors stay fixed at their target
                new_positions.push(anchor_map[&i]);
                continue;
            }

            let nbrs = &neighbors[i];
            if nbrs.is_empty() {
                new_positions.push(output.points.get(i));
                continue;
            }

            // Desired position = average(neighbors) + laplacian[i]
            let count: f64 = nbrs.len() as f64;
            let mut avg: [f64; 3] = [0.0, 0.0, 0.0];
            for &nb in nbrs {
                let q: [f64; 3] = output.points.get(nb);
                avg[0] += q[0];
                avg[1] += q[1];
                avg[2] += q[2];
            }
            avg[0] /= count;
            avg[1] /= count;
            avg[2] /= count;

            new_positions.push([
                avg[0] + laplacian[i][0],
                avg[1] + laplacian[i][1],
                avg[2] + laplacian[i][2],
            ]);
        }

        for i in 0..n {
            output.points.set(i, new_positions[i]);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad_mesh() -> PolyData {
        // A simple quad mesh (2 triangles sharing an edge):
        //  3---2
        //  |\ /|
        //  | 4 |
        //  |/ \|
        //  0---1
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]);
        pd.polys.push_cell(&[0, 1, 4]);
        pd.polys.push_cell(&[1, 2, 4]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[3, 0, 4]);
        pd
    }

    #[test]
    fn no_anchors_returns_clone() {
        let mesh = make_quad_mesh();
        let result = laplacian_deform_simple(&mesh, &[], 10);
        for i in 0..mesh.points.len() {
            let p: [f64; 3] = mesh.points.get(i);
            let q: [f64; 3] = result.points.get(i);
            assert!((p[0] - q[0]).abs() < 1e-10);
            assert!((p[1] - q[1]).abs() < 1e-10);
            assert!((p[2] - q[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn anchors_at_original_positions_no_change() {
        let mesh = make_quad_mesh();
        // Fix all corners in place at their original positions
        let anchors: Vec<(usize, [f64; 3])> = vec![
            (0, [0.0, 0.0, 0.0]),
            (1, [1.0, 0.0, 0.0]),
            (2, [1.0, 1.0, 0.0]),
            (3, [0.0, 1.0, 0.0]),
        ];
        let result = laplacian_deform_simple(&mesh, &anchors, 50);
        // Center vertex should stay near its original position
        let center: [f64; 3] = result.points.get(4);
        assert!((center[0] - 0.5).abs() < 0.1);
        assert!((center[1] - 0.5).abs() < 0.1);
        assert!((center[2]).abs() < 0.1);
    }

    #[test]
    fn moving_anchor_displaces_neighbors() {
        let mesh = make_quad_mesh();
        // Fix corners 0,1,2,3 in place, move center vertex 4 upward
        let anchors: Vec<(usize, [f64; 3])> = vec![
            (0, [0.0, 0.0, 0.0]),
            (1, [1.0, 0.0, 0.0]),
            (2, [1.0, 1.0, 0.0]),
            (3, [0.0, 1.0, 0.0]),
            (4, [0.5, 0.5, 1.0]),
        ];
        let result = laplacian_deform_simple(&mesh, &anchors, 10);
        // Center vertex should be at the target
        let center: [f64; 3] = result.points.get(4);
        assert!((center[0] - 0.5).abs() < 1e-10);
        assert!((center[1] - 0.5).abs() < 1e-10);
        assert!((center[2] - 1.0).abs() < 1e-10);
    }
}
