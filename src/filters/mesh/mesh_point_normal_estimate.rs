//! Estimate normals for unstructured point clouds using PCA of local neighborhoods.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate normals for a point cloud using K nearest neighbors.
pub fn estimate_normals_knn(mesh: &PolyData, k: usize) -> PolyData {
    let n = mesh.points.len();
    let k = k.max(1).min(n);
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut normals = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = pts[i];
        // Find K nearest neighbors (brute force), including the query point.
        let mut dists: Vec<(usize, f64)> = (0..n)
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
                d2 <= r2
            })
            .map(|j| pts[j])
            .collect();

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

fn pca_normal(points: &[[f64; 3]]) -> [f64; 3] {
    let n = points.len() as f64;
    if n == 0.0 {
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

    smallest_eigenvector(cov)
}

fn smallest_eigenvector(mut a: [[f64; 3]; 3]) -> [f64; 3] {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..50 {
        let mut p = 0;
        let mut q = 1;
        let mut max_value = 0.0;
        for row in 0..3 {
            for col in (row + 1)..3 {
                let value = a[row][col].abs();
                if value > max_value {
                    max_value = value;
                    p = row;
                    q = col;
                }
            }
        }
        if max_value < 1e-15 {
            break;
        }

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        let theta = if (app - aqq).abs() < 1e-20 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };
        let c = theta.cos();
        let s = theta.sin();

        let mut next = a;
        next[p][p] = c * c * app + 2.0 * c * s * apq + s * s * aqq;
        next[q][q] = s * s * app - 2.0 * c * s * apq + c * c * aqq;
        next[p][q] = 0.0;
        next[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                next[r][p] = c * arp + s * arq;
                next[p][r] = next[r][p];
                next[r][q] = -s * arp + c * arq;
                next[q][r] = next[r][q];
            }
        }
        a = next;

        for row in &mut v {
            let vp = row[p];
            let vq = row[q];
            row[p] = c * vp + s * vq;
            row[q] = -s * vp + c * vq;
        }
    }

    let mut min_index = 0;
    for i in 1..3 {
        if a[i][i] < a[min_index][min_index] {
            min_index = i;
        }
    }

    let normal = [v[0][min_index], v[1][min_index], v[2][min_index]];
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [normal[0] / len, normal[1] / len, normal[2] / len]
    }
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
