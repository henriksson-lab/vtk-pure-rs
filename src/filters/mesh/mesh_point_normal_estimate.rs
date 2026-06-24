//! Estimate normals for unstructured point clouds using PCA of local neighborhoods.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate normals for a point cloud using K nearest neighbors.
pub fn estimate_normals_knn(mesh: &PolyData, k: usize) -> PolyData {
    let n = mesh.points.len();
    let k = k.max(3).min(n);
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut normals = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = pts[i];
        // Find K nearest neighbors (brute force)
        let mut dists: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                let d2 = (pts[j][0] - p[0]).powi(2)
                    + (pts[j][1] - p[1]).powi(2)
                    + (pts[j][2] - p[2]).powi(2);
                (j, d2)
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let neighbors: Vec<[f64; 3]> = dists.iter().take(k).map(|&(j, _)| pts[j]).collect();

        let normal = pca_normal(&neighbors);
        normals.extend_from_slice(&normal);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    result.point_data_mut().set_active_normals("Normals");
    result
}

/// Estimate normals using radius search.
pub fn estimate_normals_radius(mesh: &PolyData, radius: f64) -> PolyData {
    let n = mesh.points.len();
    let r2 = radius * radius;
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut normals = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = pts[i];
        let neighbors: Vec<[f64; 3]> = (0..n)
            .filter(|&j| {
                let d2 = (pts[j][0] - p[0]).powi(2)
                    + (pts[j][1] - p[1]).powi(2)
                    + (pts[j][2] - p[2]).powi(2);
                d2 <= r2 && j != i
            })
            .map(|j| pts[j])
            .collect();

        let normal = if neighbors.len() >= 2 {
            pca_normal(&neighbors)
        } else {
            [0.0, 0.0, 1.0]
        };
        normals.extend_from_slice(&normal);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    result.point_data_mut().set_active_normals("Normals");
    result
}

fn pca_normal(points: &[[f64; 3]]) -> [f64; 3] {
    let n = points.len() as f64;
    if n < 2.0 {
        return [0.0, 0.0, 1.0];
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for p in points {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n;
    cy /= n;
    cz /= n;

    // Covariance matrix
    let mut cov = [[0.0f64; 3]; 3];
    for p in points {
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }

    // Power iteration for smallest eigenvector
    let mut v = [1.0, 0.0, 0.0];
    for _ in 0..30 {
        // Apply inverse-ish: find direction least amplified
        let mv = mat_vec(&cov, v);
        let len = (mv[0] * mv[0] + mv[1] * mv[1] + mv[2] * mv[2]).sqrt();
        if len < 1e-15 {
            break;
        }
        v = [mv[0] / len, mv[1] / len, mv[2] / len];
    }
    // v is now the largest eigenvector. We want the smallest.
    // Use cross product of two largest eigenvectors approximation
    let v1 = v;
    let mut v2 = if v1[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    // Gram-Schmidt
    let dot = v2[0] * v1[0] + v2[1] * v1[1] + v2[2] * v1[2];
    v2 = [
        v2[0] - dot * v1[0],
        v2[1] - dot * v1[1],
        v2[2] - dot * v1[2],
    ];
    let len = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    v2 = [v2[0] / len, v2[1] / len, v2[2] / len];
    for _ in 0..30 {
        let mv = mat_vec(&cov, v2);
        let d1 = mv[0] * v1[0] + mv[1] * v1[1] + mv[2] * v1[2];
        let mv2 = [mv[0] - d1 * v1[0], mv[1] - d1 * v1[1], mv[2] - d1 * v1[2]];
        let len = (mv2[0] * mv2[0] + mv2[1] * mv2[1] + mv2[2] * mv2[2]).sqrt();
        if len < 1e-15 {
            break;
        }
        v2 = [mv2[0] / len, mv2[1] / len, mv2[2] / len];
    }
    // Normal = v1 x v2
    let normal = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [normal[0] / len, normal[1] / len, normal[2] / len]
    }
}

fn mat_vec(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_knn() {
        // Points on Z=0 plane -> normal should be ~(0,0,±1)
        let mut mesh = PolyData::new();
        for i in 0..5 {
            for j in 0..5 {
                mesh.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let r = estimate_normals_knn(&mesh, 5);
        let arr = r.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(12, &mut buf); // center point
        assert!(buf[2].abs() > 0.9, "normal z = {}", buf[2]);
    }
    #[test]
    fn test_radius() {
        let mut mesh = PolyData::new();
        for i in 0..4 {
            for j in 0..4 {
                mesh.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let r = estimate_normals_radius(&mesh, 2.0);
        assert!(r.point_data().get_array("Normals").is_some());
    }
}
