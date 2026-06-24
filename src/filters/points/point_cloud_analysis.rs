//! Point cloud analysis: bounding sphere, covariance, planarity.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the bounding sphere of a point cloud.
///
/// Returns (center, radius).
pub fn bounding_sphere(mesh: &PolyData) -> ([f64; 3], f64) {
    let n = mesh.points.len();
    if n == 0 {
        return ([0.0; 3], 0.0);
    }

    // Compute centroid
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    for j in 0..3 {
        c[j] /= n as f64;
    }

    // Find max distance from centroid
    let mut max_r2 = 0.0f64;
    for i in 0..n {
        let p = mesh.points.get(i);
        let r2 = (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
        max_r2 = max_r2.max(r2);
    }

    (c, max_r2.sqrt())
}

/// Compute planarity score (0=3D, 1=perfectly planar).
pub fn planarity(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n < 3 {
        return 1.0;
    }

    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    for j in 0..3 {
        c[j] /= n as f64;
    }

    // Covariance matrix
    let mut cov = [[0.0; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for r in 0..3 {
            for cc in 0..3 {
                cov[r][cc] += d[r] * d[cc];
            }
        }
    }
    for r in 0..3 {
        for cc in 0..3 {
            cov[r][cc] /= n as f64;
        }
    }

    // Eigenvalues via characteristic polynomial (3x3)
    let eigenvalues = eigenvalues_3x3(&cov);
    let mut ev = eigenvalues;
    ev.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let total = ev[0] + ev[1] + ev[2];
    if total < 1e-15 {
        return 1.0;
    }
    1.0 - ev[2] / total // small third eigenvalue → planar
}

/// Compute linearity score (0=3D, 1=perfectly linear).
pub fn linearity(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n < 2 {
        return 1.0;
    }

    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    for j in 0..3 {
        c[j] /= n as f64;
    }

    let mut cov = [[0.0; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for r in 0..3 {
            for cc in 0..3 {
                cov[r][cc] += d[r] * d[cc];
            }
        }
    }

    let eigenvalues = eigenvalues_3x3(&cov);
    let mut ev = eigenvalues;
    ev.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    if ev[0] < 1e-15 {
        return 0.0;
    }
    (ev[0] - ev[1]) / ev[0]
}

/// Add local density and curvature estimates per point.
pub fn point_cloud_features(mesh: &PolyData, k: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 2 {
        return mesh.clone();
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut density = Vec::with_capacity(n);
    let mut roughness = Vec::with_capacity(n);

    for i in 0..n {
        let mut dists: Vec<(usize, f64)> = pts
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(j, p)| {
                (
                    j,
                    (p[0] - pts[i][0]).powi(2)
                        + (p[1] - pts[i][1]).powi(2)
                        + (p[2] - pts[i][2]).powi(2),
                )
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let knn: Vec<usize> = dists.iter().take(k).map(|(j, _)| *j).collect();
        let k_dist = if !dists.is_empty() && dists.len() >= k {
            dists[k - 1].1.sqrt()
        } else {
            1.0
        };
        density.push(k as f64 / (std::f64::consts::PI * k_dist * k_dist + 1e-15));

        // Roughness: deviation from local plane
        if knn.len() >= 3 {
            let mut avg = [0.0; 3];
            for &j in &knn {
                for c in 0..3 {
                    avg[c] += pts[j][c];
                }
            }
            for c in 0..3 {
                avg[c] /= knn.len() as f64;
            }
            let dev = ((pts[i][0] - avg[0]).powi(2)
                + (pts[i][1] - avg[1]).powi(2)
                + (pts[i][2] - avg[2]).powi(2))
            .sqrt();
            roughness.push(dev);
        } else {
            roughness.push(0.0);
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LocalDensity",
            density,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Roughness",
            roughness,
            1,
        )));
    result
}

fn eigenvalues_3x3(m: &[[f64; 3]; 3]) -> [f64; 3] {
    let p1 = m[0][1] * m[0][1] + m[0][2] * m[0][2] + m[1][2] * m[1][2];
    if p1 < 1e-20 {
        return [m[0][0], m[1][1], m[2][2]];
    }

    let q = (m[0][0] + m[1][1] + m[2][2]) / 3.0;
    let p2 = (m[0][0] - q).powi(2) + (m[1][1] - q).powi(2) + (m[2][2] - q).powi(2) + 2.0 * p1;
    let p = (p2 / 6.0).sqrt();

    let b = [
        [(m[0][0] - q) / p, m[0][1] / p, m[0][2] / p],
        [m[1][0] / p, (m[1][1] - q) / p, m[1][2] / p],
        [m[2][0] / p, m[2][1] / p, (m[2][2] - q) / p],
    ];

    let det_b = b[0][0] * (b[1][1] * b[2][2] - b[1][2] * b[2][1])
        - b[0][1] * (b[1][0] * b[2][2] - b[1][2] * b[2][0])
        + b[0][2] * (b[1][0] * b[2][1] - b[1][1] * b[2][0]);

    let r = det_b / 2.0;
    let phi = if r <= -1.0 {
        std::f64::consts::PI / 3.0
    } else if r >= 1.0 {
        0.0
    } else {
        r.acos() / 3.0
    };

    let e1 = q + 2.0 * p * phi.cos();
    let e3 = q + 2.0 * p * (phi + 2.0 * std::f64::consts::PI / 3.0).cos();
    let e2 = 3.0 * q - e1 - e3;
    [e1, e2, e3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_bounds() {
        let pts: Vec<[f64; 3]> = (0..100)
            .map(|i| {
                let t = i as f64 * 0.1;
                [t.cos(), t.sin(), 0.0]
            })
            .collect();
        let mesh = PolyData::from_points(pts);
        let (center, radius) = bounding_sphere(&mesh);
        assert!(radius > 0.0 && radius <= 2.0);
    }

    #[test]
    fn planar_points() {
        let pts: Vec<[f64; 3]> = (0..50)
            .map(|i| [i as f64, (i * 3 % 20) as f64, 0.0])
            .collect();
        let mesh = PolyData::from_points(pts);
        assert!(planarity(&mesh) > 0.95);
    }

    #[test]
    fn linear_points() {
        let pts: Vec<[f64; 3]> = (0..50).map(|i| [i as f64, 0.0, 0.0]).collect();
        let mesh = PolyData::from_points(pts);
        assert!(linearity(&mesh) > 0.95);
    }

    #[test]
    fn features() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        let result = point_cloud_features(&mesh, 2);
        assert!(result.point_data().get_array("LocalDensity").is_some());
        assert!(result.point_data().get_array("Roughness").is_some());
    }
}
