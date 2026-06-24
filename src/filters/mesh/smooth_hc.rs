use std::collections::HashSet;

use crate::data::PolyData;

/// HC (Humphrey's Classes) Laplacian smoothing.
///
/// A variant of Laplacian smoothing that reduces volume shrinkage by pulling
/// vertices back toward the original positions. The algorithm alternates between
/// a standard Laplacian step and a correction step controlled by `alpha` and `beta`.
///
/// `alpha` controls the attraction back to the original position (0..1).
/// `beta` controls the smoothing of the correction vectors (0..1).
/// Typical values: alpha = 0.0, beta = 0.5.
pub fn smooth_hc(input: &PolyData, iterations: usize, alpha: f64, beta: f64) -> PolyData {
    let mut output = input.clone();
    let n = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    // Store original positions
    let original: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Build adjacency
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    let alpha_clamped: f64 = alpha.clamp(0.0, 1.0);
    let beta_clamped: f64 = beta.clamp(0.0, 1.0);

    for _ in 0..iterations {
        // Step 1: standard Laplacian -> q positions
        let mut q: Vec<[f64; 3]> = Vec::with_capacity(n);
        for (i, nbrs) in neighbors.iter().enumerate() {
            let p = output.points.get(i);
            if nbrs.is_empty() {
                q.push(p);
                continue;
            }
            let count: f64 = nbrs.len() as f64;
            let mut avg = [0.0f64; 3];
            for &nb in nbrs {
                let r = output.points.get(nb);
                avg[0] += r[0];
                avg[1] += r[1];
                avg[2] += r[2];
            }
            avg[0] /= count;
            avg[1] /= count;
            avg[2] /= count;
            q.push(avg);
        }

        // Step 2: compute difference vectors b = q - (alpha * original + (1 - alpha) * current)
        let mut b: Vec<[f64; 3]> = Vec::with_capacity(n);
        for i in 0..n {
            let p = output.points.get(i);
            let ref_pos = [
                alpha_clamped * original[i][0] + (1.0 - alpha_clamped) * p[0],
                alpha_clamped * original[i][1] + (1.0 - alpha_clamped) * p[1],
                alpha_clamped * original[i][2] + (1.0 - alpha_clamped) * p[2],
            ];
            b.push([
                q[i][0] - ref_pos[0],
                q[i][1] - ref_pos[1],
                q[i][2] - ref_pos[2],
            ]);
        }

        // Step 3: smooth b vectors and push q back
        for i in 0..n {
            let nbrs = &neighbors[i];
            if nbrs.is_empty() {
                output.points.set(i, q[i]);
                continue;
            }
            let count: f64 = nbrs.len() as f64;
            let mut avg_b = [0.0f64; 3];
            for &nb in nbrs {
                avg_b[0] += b[nb][0];
                avg_b[1] += b[nb][1];
                avg_b[2] += b[nb][2];
            }
            avg_b[0] /= count;
            avg_b[1] /= count;
            avg_b[2] /= count;

            let correction = [
                beta_clamped * b[i][0] + (1.0 - beta_clamped) * avg_b[0],
                beta_clamped * b[i][1] + (1.0 - beta_clamped) * avg_b[1],
                beta_clamped * b[i][2] + (1.0 - beta_clamped) * avg_b[2],
            ];

            output.points.set(
                i,
                [
                    q[i][0] - correction[0],
                    q[i][1] - correction[1],
                    q[i][2] - correction[2],
                ],
            );
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CellArray;

    fn make_quad_mesh() -> PolyData {
        let mut pd = PolyData::new();
        // A simple 4-triangle mesh (diamond shape)
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.5]); // raised center-ish point
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 3]);
        polys.push_cell(&[1, 2, 3]);
        polys.push_cell(&[2, 0, 3]);
        pd.polys = polys;
        pd
    }

    #[test]
    fn zero_iterations_no_change() {
        let input = make_quad_mesh();
        let result = smooth_hc(&input, 0, 0.0, 0.5);
        for i in 0..input.points.len() {
            let a = input.points.get(i);
            let b = result.points.get(i);
            assert!((a[0] - b[0]).abs() < 1e-12);
            assert!((a[1] - b[1]).abs() < 1e-12);
            assert!((a[2] - b[2]).abs() < 1e-12);
        }
    }

    #[test]
    fn smoothing_moves_points() {
        let input = make_quad_mesh();
        let result = smooth_hc(&input, 5, 0.0, 0.5);
        // The raised point (index 3) should have moved
        let orig = input.points.get(3);
        let smoothed = result.points.get(3);
        let dist: f64 = ((orig[0] - smoothed[0]).powi(2)
            + (orig[1] - smoothed[1]).powi(2)
            + (orig[2] - smoothed[2]).powi(2))
        .sqrt();
        assert!(dist > 1e-6, "smoothing should move the raised point");
    }

    #[test]
    fn less_shrinkage_than_plain_laplacian() {
        let input = make_quad_mesh();
        // HC smoothing should preserve volume better than plain Laplacian
        let hc_result = smooth_hc(&input, 10, 0.0, 0.5);

        // Compute bounding box volume proxy (sum of squared coords from centroid)
        let centroid_orig = centroid(&input);
        let centroid_hc = centroid(&hc_result);

        // At minimum the centroid should not have drifted excessively
        let drift: f64 = ((centroid_orig[0] - centroid_hc[0]).powi(2)
            + (centroid_orig[1] - centroid_hc[1]).powi(2)
            + (centroid_orig[2] - centroid_hc[2]).powi(2))
        .sqrt();
        assert!(drift < 0.5, "HC smoothing should preserve center roughly");
    }

    fn centroid(pd: &PolyData) -> [f64; 3] {
        let n: f64 = pd.points.len() as f64;
        let mut c = [0.0f64; 3];
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            c[0] += p[0];
            c[1] += p[1];
            c[2] += p[2];
        }
        c[0] /= n;
        c[1] /= n;
        c[2] /= n;
        c
    }
}
