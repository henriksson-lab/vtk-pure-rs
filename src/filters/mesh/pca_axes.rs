use crate::data::PolyData;

/// Result of PCA on mesh point positions.
#[derive(Debug, Clone)]
pub struct PcaResult {
    /// Centroid of the point cloud.
    pub centroid: [f64; 3],
    /// Principal axes as unit vectors, ordered by decreasing eigenvalue.
    /// axes[0] is the direction of greatest variance.
    pub axes: [[f64; 3]; 3],
    /// Eigenvalues in decreasing order (variance along each axis).
    pub eigenvalues: [f64; 3],
}

/// Compute PCA (principal component analysis) of point positions in a PolyData.
///
/// Returns the 3 principal axes and eigenvalues derived from the covariance
/// matrix of the point cloud, using a closed-form 3x3 symmetric eigensolver.
pub fn compute_pca(input: &PolyData) -> PcaResult {
    let n: usize = input.points.len();

    if n == 0 {
        return PcaResult {
            centroid: [0.0, 0.0, 0.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            eigenvalues: [0.0, 0.0, 0.0],
        };
    }

    // Compute centroid
    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let inv_n: f64 = 1.0 / n as f64;
    cx *= inv_n;
    cy *= inv_n;
    cz *= inv_n;

    // Compute covariance matrix (symmetric 3x3)
    let mut cov = [0.0f64; 6]; // xx, xy, xz, yy, yz, zz
    for i in 0..n {
        let p = input.points.get(i);
        let dx: f64 = p[0] - cx;
        let dy: f64 = p[1] - cy;
        let dz: f64 = p[2] - cz;
        cov[0] += dx * dx;
        cov[1] += dx * dy;
        cov[2] += dx * dz;
        cov[3] += dy * dy;
        cov[4] += dy * dz;
        cov[5] += dz * dz;
    }
    for v in cov.iter_mut() {
        *v *= inv_n;
    }

    // Eigendecomposition of 3x3 symmetric matrix using Cardano's method
    let (eigenvalues, axes) = symmetric_eigen_3x3(cov[0], cov[1], cov[2], cov[3], cov[4], cov[5]);

    PcaResult {
        centroid: [cx, cy, cz],
        axes,
        eigenvalues,
    }
}

/// Solve eigenvalues of a 3x3 symmetric matrix using Cardano's analytical method.
/// Matrix layout:
/// | a  b  c |
/// | b  d  e |
/// | c  e  f |
///
/// Returns (eigenvalues sorted descending, corresponding eigenvectors).
fn symmetric_eigen_3x3(
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
) -> ([f64; 3], [[f64; 3]; 3]) {
    // Characteristic equation: det(A - lI) = 0
    // -l^3 + (a+d+f)l^2 - (ad+af+df-b^2-c^2-e^2)l + det(A) = 0
    let p1: f64 = b * b + c * c + e * e;

    if p1 < 1e-30 {
        // Matrix is diagonal
        let mut evals = [a, d, f];
        let mut evecs: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        // Sort descending
        for i in 0..3 {
            for j in (i + 1)..3 {
                if evals[j] > evals[i] {
                    evals.swap(i, j);
                    evecs.swap(i, j);
                }
            }
        }
        return (evals, evecs);
    }

    let q: f64 = (a + d + f) / 3.0;
    let p2: f64 = (a - q) * (a - q) + (d - q) * (d - q) + (f - q) * (f - q) + 2.0 * p1;
    let p: f64 = (p2 / 6.0).sqrt();

    // B = (1/p)(A - qI)
    let inv_p: f64 = 1.0 / p;
    let b11: f64 = (a - q) * inv_p;
    let b12: f64 = b * inv_p;
    let b13: f64 = c * inv_p;
    let b22: f64 = (d - q) * inv_p;
    let b23: f64 = e * inv_p;
    let b33: f64 = (f - q) * inv_p;

    // det(B)
    let det_b: f64 = b11 * (b22 * b33 - b23 * b23) - b12 * (b12 * b33 - b23 * b13)
        + b13 * (b12 * b23 - b22 * b13);

    let half_det: f64 = det_b / 2.0;
    let half_det_clamped: f64 = half_det.max(-1.0).min(1.0);

    let phi: f64 = half_det_clamped.acos() / 3.0;
    let two_p: f64 = 2.0 * p;

    let pi: f64 = std::f64::consts::PI;
    let mut evals: [f64; 3] = [
        q + two_p * phi.cos(),
        q + two_p * (phi + 2.0 * pi / 3.0).cos(),
        q + two_p * (phi + 4.0 * pi / 3.0).cos(),
    ];

    // Sort descending
    if evals[1] > evals[0] {
        evals.swap(0, 1);
    }
    if evals[2] > evals[0] {
        evals.swap(0, 2);
    }
    if evals[2] > evals[1] {
        evals.swap(1, 2);
    }

    // Compute eigenvectors
    let mut evecs: [[f64; 3]; 3] = [[0.0; 3]; 3];
    for (idx, &lam) in evals.iter().enumerate() {
        evecs[idx] = compute_eigenvector(a - lam, b, c, b, d - lam, e, c, e, f - lam);
    }

    // Ensure orthogonality: re-derive third vector via cross product
    evecs[2] = cross(evecs[0], evecs[1]);
    let len2: f64 = dot(evecs[2], evecs[2]);
    if len2 > 1e-30 {
        let inv: f64 = 1.0 / len2.sqrt();
        evecs[2][0] *= inv;
        evecs[2][1] *= inv;
        evecs[2][2] *= inv;
    }

    (evals, evecs)
}

fn compute_eigenvector(
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f_val: f64,
    g: f64,
    h: f64,
    i: f64,
) -> [f64; 3] {
    // Use cross products of rows to find the eigenvector
    let r0 = [a, b, c];
    let r1 = [d, e, f_val];
    let r2 = [g, h, i];

    let c01 = cross(r0, r1);
    let c02 = cross(r0, r2);
    let c12 = cross(r1, r2);

    let d01: f64 = dot(c01, c01);
    let d02: f64 = dot(c02, c02);
    let d12: f64 = dot(c12, c12);

    let best = if d01 >= d02 && d01 >= d12 {
        c01
    } else if d02 >= d12 {
        c02
    } else {
        c12
    };

    let len: f64 = dot(best, best).sqrt();
    if len > 1e-30 {
        [best[0] / len, best[1] / len, best[2] / len]
    } else {
        [1.0, 0.0, 0.0]
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_aligned_points() {
        // Points spread along X axis -> first PCA axis should be ~X
        let mut pd = PolyData::new();
        for i in 0..100 {
            let x: f64 = i as f64;
            pd.points.push([x, 0.0, 0.0]);
        }
        let result = compute_pca(&pd);
        // First eigenvalue should be largest
        assert!(result.eigenvalues[0] >= result.eigenvalues[1]);
        assert!(result.eigenvalues[1] >= result.eigenvalues[2]);
        // First axis should be close to +/- X
        assert!(result.axes[0][0].abs() > 0.99);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = compute_pca(&pd);
        assert_eq!(result.eigenvalues, [0.0, 0.0, 0.0]);
        assert_eq!(result.centroid, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn two_axis_spread() {
        // Points spread in XY plane -> third eigenvalue should be ~0
        let mut pd = PolyData::new();
        for i in 0..10 {
            for j in 0..10 {
                let x: f64 = i as f64;
                let y: f64 = j as f64 * 0.5;
                pd.points.push([x, y, 0.0]);
            }
        }
        let result = compute_pca(&pd);
        // Third eigenvalue should be near zero (no spread in Z)
        assert!(result.eigenvalues[2].abs() < 1e-10);
        // First eigenvalue > second (X spread is larger)
        assert!(result.eigenvalues[0] > result.eigenvalues[1]);
    }
}
