use crate::data::PolyData;

/// Result of ICP (Iterative Closest Point) registration.
#[derive(Debug, Clone)]
pub struct IcpResult {
    /// The 4×4 transformation matrix (row-major) that best aligns source to target.
    pub transform: [[f64; 4]; 4],
    /// Root mean square error of the final alignment.
    pub rms_error: f64,
    /// Number of iterations performed.
    pub iterations: usize,
}

/// Align source PolyData to target using Iterative Closest Point (ICP).
///
/// Uses a simple point-to-point ICP with SVD-based rigid body estimation.
/// Returns the 4×4 transformation matrix that aligns source to target.
pub fn icp(
    source: &PolyData,
    target: &PolyData,
    max_iterations: usize,
    tolerance: f64,
) -> IcpResult {
    let n = source.points.len();
    if n == 0 || target.points.is_empty() {
        return IcpResult {
            transform: identity_4x4(),
            rms_error: 0.0,
            iterations: 0,
        };
    }

    let target_pts: Vec<[f64; 3]> = (0..target.points.len())
        .map(|i| target.points.get(i))
        .collect();

    let mut src_pts: Vec<[f64; 3]> = (0..n).map(|i| source.points.get(i)).collect();
    let mut total_transform = identity_4x4();
    let mut prev_error = f64::MAX;

    for iter in 0..max_iterations {
        // Find closest points in target for each source point
        let correspondences: Vec<[f64; 3]> = src_pts
            .iter()
            .map(|sp| nearest_point(sp, &target_pts))
            .collect();

        // Compute centroids
        let src_centroid = centroid(&src_pts);
        let tgt_centroid = centroid(&correspondences);

        // Compute cross-covariance matrix H
        let mut h = [[0.0f64; 3]; 3];
        for i in 0..n {
            let s = [
                src_pts[i][0] - src_centroid[0],
                src_pts[i][1] - src_centroid[1],
                src_pts[i][2] - src_centroid[2],
            ];
            let t = [
                correspondences[i][0] - tgt_centroid[0],
                correspondences[i][1] - tgt_centroid[1],
                correspondences[i][2] - tgt_centroid[2],
            ];
            for r in 0..3 {
                for c in 0..3 {
                    h[r][c] += s[r] * t[c];
                }
            }
        }

        // SVD via power iteration (simplified — finds best rotation)
        let (u, vt) = svd_3x3_approx(&h);
        let mut r = mat_mul_3x3(&transpose_3x3(&vt), &transpose_3x3(&u));

        // Ensure proper rotation (det = +1)
        let det = det_3x3(&r);
        if det < 0.0 {
            for row in &mut r {
                row[2] = -row[2];
            }
        }

        // Compute translation
        let t = [
            tgt_centroid[0]
                - (r[0][0] * src_centroid[0]
                    + r[0][1] * src_centroid[1]
                    + r[0][2] * src_centroid[2]),
            tgt_centroid[1]
                - (r[1][0] * src_centroid[0]
                    + r[1][1] * src_centroid[1]
                    + r[1][2] * src_centroid[2]),
            tgt_centroid[2]
                - (r[2][0] * src_centroid[0]
                    + r[2][1] * src_centroid[1]
                    + r[2][2] * src_centroid[2]),
        ];

        // Apply transform to source points
        for p in &mut src_pts {
            let x = r[0][0] * p[0] + r[0][1] * p[1] + r[0][2] * p[2] + t[0];
            let y = r[1][0] * p[0] + r[1][1] * p[1] + r[1][2] * p[2] + t[1];
            let z = r[2][0] * p[0] + r[2][1] * p[1] + r[2][2] * p[2] + t[2];
            *p = [x, y, z];
        }

        // Accumulate transform
        let step = [
            [r[0][0], r[0][1], r[0][2], t[0]],
            [r[1][0], r[1][1], r[1][2], t[1]],
            [r[2][0], r[2][1], r[2][2], t[2]],
            [0.0, 0.0, 0.0, 1.0],
        ];
        total_transform = mul_4x4(&step, &total_transform);

        // Compute RMS error
        let mut sum_d2 = 0.0;
        for i in 0..n {
            let d = [
                src_pts[i][0] - correspondences[i][0],
                src_pts[i][1] - correspondences[i][1],
                src_pts[i][2] - correspondences[i][2],
            ];
            sum_d2 += d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        }
        let rms = (sum_d2 / n as f64).sqrt();

        if (prev_error - rms).abs() < tolerance {
            return IcpResult {
                transform: total_transform,
                rms_error: rms,
                iterations: iter + 1,
            };
        }
        prev_error = rms;
    }

    IcpResult {
        transform: total_transform,
        rms_error: prev_error,
        iterations: max_iterations,
    }
}

fn nearest_point(query: &[f64; 3], points: &[[f64; 3]]) -> [f64; 3] {
    let mut best = points[0];
    let mut best_d = f64::MAX;
    for p in points {
        let d = (query[0] - p[0]).powi(2) + (query[1] - p[1]).powi(2) + (query[2] - p[2]).powi(2);
        if d < best_d {
            best_d = d;
            best = *p;
        }
    }
    best
}

fn centroid(pts: &[[f64; 3]]) -> [f64; 3] {
    let n = pts.len() as f64;
    let mut c = [0.0; 3];
    for p in pts {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    [c[0] / n, c[1] / n, c[2] / n]
}

fn identity_4x4() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mul_4x4(a: &[[f64; 4]; 4], b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

fn transpose_3x3(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

fn det_3x3(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn mat_mul_3x3(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

/// Approximate SVD of a 3×3 matrix using Jacobi iterations.
fn svd_3x3_approx(m: &[[f64; 3]; 3]) -> ([[f64; 3]; 3], [[f64; 3]; 3]) {
    // Compute M^T * M
    let mtm = mat_mul_3x3(&transpose_3x3(m), m);

    // Power iteration to find dominant eigenvector of M^T M
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    // Simple QR-like iteration
    for _ in 0..30 {
        let av = mat_mul_3x3(&mtm, &v);
        v = gram_schmidt(&av);
    }

    // U = M * V * Sigma^-1 (approximated)
    let mv = mat_mul_3x3(m, &v);
    let u = gram_schmidt(&mv);

    (u, transpose_3x3(&v))
}

fn gram_schmidt(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut u = *m;
    // Normalize first column
    let l0 = (u[0][0] * u[0][0] + u[1][0] * u[1][0] + u[2][0] * u[2][0]).sqrt();
    if l0 > 1e-20 {
        for row in &mut u {
            row[0] /= l0;
        }
    }
    // Orthogonalize and normalize second column
    let d01: f64 = (0..3).map(|i| u[i][0] * u[i][1]).sum();
    for row in &mut u {
        row[1] -= d01 * row[0];
    }
    let l1 = (u[0][1] * u[0][1] + u[1][1] * u[1][1] + u[2][1] * u[2][1]).sqrt();
    if l1 > 1e-20 {
        for row in &mut u {
            row[1] /= l1;
        }
    }
    // Third column = cross product
    u[0][2] = u[1][0] * u[2][1] - u[2][0] * u[1][1];
    u[1][2] = u[2][0] * u[0][1] - u[0][0] * u[2][1];
    u[2][2] = u[0][0] * u[1][1] - u[1][0] * u[0][1];
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icp_identity() {
        // Source and target are the same — should converge with identity transform
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        let result = icp(&pd, &pd, 10, 1e-10);
        assert!(result.rms_error < 1e-6);
    }

    #[test]
    fn icp_translation() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([1.0, 0.0, 0.0]);
        source.points.push([0.0, 1.0, 0.0]);
        source.points.push([1.0, 1.0, 0.0]);

        let mut target = PolyData::new();
        target.points.push([1.0, 0.0, 0.0]);
        target.points.push([2.0, 0.0, 0.0]);
        target.points.push([1.0, 1.0, 0.0]);
        target.points.push([2.0, 1.0, 0.0]);

        let result = icp(&source, &target, 50, 1e-10);
        assert!(result.rms_error < 0.1, "rms = {}", result.rms_error);
        // Translation should be approximately [1, 0, 0]
        assert!(
            (result.transform[0][3] - 1.0).abs() < 0.2,
            "tx = {}",
            result.transform[0][3]
        );
    }
}
