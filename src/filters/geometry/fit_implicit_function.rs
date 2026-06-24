//! Fit implicit surface to a point cloud.
//!
//! Fits a quadric surface (ax² + by² + cz² + dxy + exz + fyz + gx + hy + iz + j = 0)
//! to a set of 3D points using least-squares.

use crate::data::PolyData;

/// Coefficients of a quadric surface:
/// a*x² + b*y² + c*z² + d*xy + e*xz + f*yz + g*x + h*y + i*z + j = 0
#[derive(Debug, Clone)]
pub struct QuadricFit {
    pub coeffs: [f64; 10], // [a, b, c, d, e, f, g, h, i, j]
    pub residual: f64,
}

impl QuadricFit {
    /// Evaluate the quadric at a point.
    pub fn evaluate(&self, x: f64, y: f64, z: f64) -> f64 {
        let c = &self.coeffs;
        c[0] * x * x
            + c[1] * y * y
            + c[2] * z * z
            + c[3] * x * y
            + c[4] * x * z
            + c[5] * y * z
            + c[6] * x
            + c[7] * y
            + c[8] * z
            + c[9]
    }
}

/// Fit a quadric surface to a point cloud using least-squares.
///
/// Minimizes sum of (f(p_i))^2 subject to ||coeffs|| = 1.
/// Returns the fitted quadric coefficients.
pub fn fit_quadric(points: &PolyData) -> Option<QuadricFit> {
    let n = points.points.len();
    if n < 10 {
        return None;
    } // need at least 10 points for 10 coefficients

    // Build the design matrix A where each row is [x², y², z², xy, xz, yz, x, y, z, 1]
    let mut ata = [[0.0f64; 10]; 10]; // A^T * A

    for i in 0..n {
        let p = points.points.get(i);
        let x = p[0];
        let y = p[1];
        let z = p[2];
        let row = [x * x, y * y, z * z, x * y, x * z, y * z, x, y, z, 1.0];

        for r in 0..10 {
            for c in 0..10 {
                ata[r][c] += row[r] * row[c];
            }
        }
    }

    // Find eigenvector of A^T*A with smallest eigenvalue using inverse power iteration
    let coeffs = smallest_eigenvector(&ata)?;

    // Compute residual
    let mut residual = 0.0;
    for i in 0..n {
        let p = points.points.get(i);
        let v = coeffs[0] * p[0] * p[0]
            + coeffs[1] * p[1] * p[1]
            + coeffs[2] * p[2] * p[2]
            + coeffs[3] * p[0] * p[1]
            + coeffs[4] * p[0] * p[2]
            + coeffs[5] * p[1] * p[2]
            + coeffs[6] * p[0]
            + coeffs[7] * p[1]
            + coeffs[8] * p[2]
            + coeffs[9];
        residual += v * v;
    }
    residual = (residual / n as f64).sqrt();

    let mut c = [0.0; 10];
    c.copy_from_slice(&coeffs[..10]);
    Some(QuadricFit {
        coeffs: c,
        residual,
    })
}

/// Fit a plane (gx + hy + iz + j = 0) to a point cloud.
pub fn fit_plane(points: &PolyData) -> Option<([f64; 3], f64)> {
    let n = points.points.len();
    if n < 3 {
        return None;
    }

    // Compute centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = points.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;

    // Covariance matrix
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = points.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }

    // Find normal as eigenvector with smallest eigenvalue (power iteration on inverse)
    // Use simplified: try coordinate axes and pick the one with smallest variance
    let mut best_normal = [0.0, 0.0, 1.0];
    let best_var = f64::MAX;

    // Power iteration for smallest eigenvector of 3x3
    let normal = smallest_eigenvector_3x3(&cov);
    let var: f64 = (0..n)
        .map(|i| {
            let p = points.points.get(i);
            let d = [p[0] - cx, p[1] - cy, p[2] - cz];
            (d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2]).powi(2)
        })
        .sum::<f64>()
        / n as f64;

    if var < best_var {
        let _ = var;
        best_normal = normal;
    }

    let d = -(best_normal[0] * cx + best_normal[1] * cy + best_normal[2] * cz);
    Some((best_normal, d))
}

fn smallest_eigenvector(m: &[[f64; 10]; 10]) -> Option<Vec<f64>> {
    // Inverse power iteration: (A - σI)^{-1} v converges to eigenvector of smallest eigenvalue
    // We use a shift σ = 0 and solve via Gauss elimination
    let n = 10;
    let mut v = vec![1.0 / (n as f64).sqrt(); n];

    for _ in 0..100 {
        // Solve A * w = v using Gaussian elimination
        let mut aug = [[0.0f64; 11]; 10];
        for r in 0..n {
            for c in 0..n {
                aug[r][c] = m[r][c];
            }
            aug[r][n] = v[r];
        }

        // Add small regularization to avoid singular matrix
        for r in 0..n {
            aug[r][r] += 1e-12;
        }

        // Forward elimination
        for col in 0..n {
            let mut max_row = col;
            for row in col + 1..n {
                if aug[row][col].abs() > aug[max_row][col].abs() {
                    max_row = row;
                }
            }
            aug.swap(col, max_row);
            if aug[col][col].abs() < 1e-15 {
                continue;
            }
            for row in col + 1..n {
                let factor = aug[row][col] / aug[col][col];
                for c in col..=n {
                    aug[row][c] -= factor * aug[col][c];
                }
            }
        }

        // Back substitution
        let mut w = vec![0.0; n];
        for row in (0..n).rev() {
            w[row] = aug[row][n];
            for c in row + 1..n {
                w[row] -= aug[row][c] * w[c];
            }
            if aug[row][row].abs() > 1e-15 {
                w[row] /= aug[row][row];
            }
        }

        // Normalize
        let norm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < 1e-15 {
            return None;
        }
        for x in &mut w {
            *x /= norm;
        }

        v = w;
    }

    Some(v)
}

fn smallest_eigenvector_3x3(m: &[[f64; 3]; 3]) -> [f64; 3] {
    // Power iteration on M to find largest eigenvector, then deflate
    let mut v = [1.0, 0.0, 0.0];
    for _ in 0..50 {
        let w = mat_vec_3(m, v);
        let norm = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if norm < 1e-15 {
            break;
        }
        v = [w[0] / norm, w[1] / norm, w[2] / norm];
    }
    let ev1 = v;
    let lambda1 = {
        let mv = mat_vec_3(m, ev1);
        mv[0] * ev1[0] + mv[1] * ev1[1] + mv[2] * ev1[2]
    };

    // Deflate
    let mut m2 = *m;
    for r in 0..3 {
        for c in 0..3 {
            m2[r][c] -= lambda1 * ev1[r] * ev1[c];
        }
    }

    // Second largest
    let mut v2 = [0.0, 1.0, 0.0];
    for _ in 0..50 {
        let w = mat_vec_3(&m2, v2);
        let norm = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if norm < 1e-15 {
            break;
        }
        v2 = [w[0] / norm, w[1] / norm, w[2] / norm];
    }

    // Smallest eigenvector is cross product of the two largest
    let n = [
        ev1[1] * v2[2] - ev1[2] * v2[1],
        ev1[2] * v2[0] - ev1[0] * v2[2],
        ev1[0] * v2[1] - ev1[1] * v2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

fn mat_vec_3(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    #[test]
    fn fit_plane_to_points() {
        // Points on z=0 plane with some noise
        let mut mesh = PolyData::new();
        let mut pts = Vec::new();
        for i in 0..20 {
            for j in 0..20 {
                pts.push([i as f64, j as f64, 0.01 * ((i + j) as f64).sin()]);
            }
        }
        mesh.points = Points::from(pts);

        let (normal, _d) = fit_plane(&mesh).unwrap();
        // Normal should be approximately [0, 0, ±1]
        assert!(normal[2].abs() > 0.9, "normal z={}", normal[2]);
    }

    #[test]
    fn fit_quadric_to_sphere() {
        let mut mesh = PolyData::new();
        let mut pts = Vec::new();
        for i in 0..30 {
            let theta = std::f64::consts::PI * i as f64 / 30.0;
            for j in 0..30 {
                let phi = 2.0 * std::f64::consts::PI * j as f64 / 30.0;
                pts.push([
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ]);
            }
        }
        mesh.points = Points::from(pts);

        let fit = fit_quadric(&mesh).unwrap();
        // For a unit sphere, x²+y²+z²-1=0, so coeffs should have
        // roughly equal x², y², z² coefficients and the rest near zero
        assert!(fit.residual < 0.1, "residual={}", fit.residual);
    }

    #[test]
    fn too_few_points() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert!(fit_quadric(&mesh).is_none());
        assert!(fit_plane(&mesh).is_none());
    }
}
