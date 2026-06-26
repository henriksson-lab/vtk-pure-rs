use std::collections::HashMap;

use crate::data::PolyData;

/// Cotangent-weighted Laplacian smoothing.
///
/// Uses cotangent weights instead of uniform weights, which better preserves
/// mesh quality and geometric features. Each vertex is displaced by
/// `lambda` times the weighted Laplacian vector per iteration.
///
/// Only triangle meshes are supported. Non-triangle cells are skipped for
/// weight computation.
pub fn smooth_cotangent(input: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let mut output = input.clone();
    let n: usize = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    for _ in 0..iterations {
        // Build cotangent weights: for each edge (i,j), accumulate cot(alpha) + cot(beta)
        // where alpha and beta are the angles opposite to edge (i,j) in the two adjacent triangles.
        let mut weights: HashMap<(usize, usize), f64> = HashMap::new();

        for cell in input.polys.iter() {
            if cell.len() != 3 {
                continue;
            }
            let Some(idx) = triangle_point_ids(cell, n) else {
                continue;
            };
            let p: [[f64; 3]; 3] = [
                output.points.get(idx[0]),
                output.points.get(idx[1]),
                output.points.get(idx[2]),
            ];

            // For each vertex k in the triangle, compute cot(angle at k)
            // and add it to the two opposite edges.
            for k in 0..3 {
                let i: usize = (k + 1) % 3;
                let j: usize = (k + 2) % 3;

                let v1: [f64; 3] = [p[i][0] - p[k][0], p[i][1] - p[k][1], p[i][2] - p[k][2]];
                let v2: [f64; 3] = [p[j][0] - p[k][0], p[j][1] - p[k][1], p[j][2] - p[k][2]];

                let dot: f64 = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
                let cross: [f64; 3] = [
                    v1[1] * v2[2] - v1[2] * v2[1],
                    v1[2] * v2[0] - v1[0] * v2[2],
                    v1[0] * v2[1] - v1[1] * v2[0],
                ];
                let cross_len: f64 =
                    (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

                let cot: f64 = if cross_len > 1e-15 {
                    dot / cross_len
                } else {
                    0.0
                };

                // Edge (i, j) is opposite to vertex k
                let edge_a: usize = idx[i].min(idx[j]);
                let edge_b: usize = idx[i].max(idx[j]);
                *weights.entry((edge_a, edge_b)).or_insert(0.0) += cot;
            }
        }

        // Build per-vertex neighbor map with weights
        let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        for (&(a, b), &w) in &weights {
            let w_clamped: f64 = w.max(0.0); // clamp negative cotangent weights
            adj[a].push((b, w_clamped));
            adj[b].push((a, w_clamped));
        }

        let mut new_positions: Vec<[f64; 3]> = Vec::with_capacity(n);

        for i in 0..n {
            let p = output.points.get(i);
            if adj[i].is_empty() {
                new_positions.push(p);
                continue;
            }

            let mut sum_w: f64 = 0.0;
            let mut lap: [f64; 3] = [0.0, 0.0, 0.0];

            for &(j, w) in &adj[i] {
                let q = output.points.get(j);
                lap[0] += w * (q[0] - p[0]);
                lap[1] += w * (q[1] - p[1]);
                lap[2] += w * (q[2] - p[2]);
                sum_w += w;
            }

            if sum_w > 1e-15 {
                let inv_w: f64 = 1.0 / sum_w;
                new_positions.push([
                    p[0] + lambda * lap[0] * inv_w,
                    p[1] + lambda * lap[1] * inv_w,
                    p[2] + lambda * lap[2] * inv_w,
                ]);
            } else {
                new_positions.push(p);
            }
        }

        for (i, pos) in new_positions.iter().enumerate() {
            output.points.set(i, *pos);
        }
    }

    output
}

fn triangle_point_ids(cell: &[i64], n: usize) -> Option<[usize; 3]> {
    Some([
        valid_point_id(cell[0], n)?,
        valid_point_id(cell[1], n)?,
        valid_point_id(cell[2], n)?,
    ])
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pyramid() -> PolyData {
        // A simple 4-triangle pyramid
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3], [0, 2, 1]],
        )
    }

    #[test]
    fn zero_iterations_no_change() {
        let pd = make_pyramid();
        let result = smooth_cotangent(&pd, 0, 0.5);
        for i in 0..pd.points.len() {
            let a = pd.points.get(i);
            let b = result.points.get(i);
            assert!((a[0] - b[0]).abs() < 1e-15);
            assert!((a[1] - b[1]).abs() < 1e-15);
            assert!((a[2] - b[2]).abs() < 1e-15);
        }
    }

    #[test]
    fn smoothing_reduces_displacement() {
        let pd = make_pyramid();
        let result = smooth_cotangent(&pd, 5, 0.3);
        // After smoothing, the apex (point 3) should move toward the base
        let original_z: f64 = pd.points.get(3)[2];
        let smoothed_z: f64 = result.points.get(3)[2];
        assert!(smoothed_z < original_z);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::from_triangles(vec![], vec![]);
        let result = smooth_cotangent(&pd, 10, 0.5);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn skips_invalid_triangles() {
        let mut pd = make_pyramid();
        pd.polys.push_cell(&[0, 1, 99]);
        pd.polys.push_cell(&[0, -1, 2]);

        let result = smooth_cotangent(&pd, 1, 0.2);
        assert_eq!(result.points.len(), pd.points.len());
    }
}
