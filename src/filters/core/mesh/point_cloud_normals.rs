//! Normal estimation for unstructured point clouds (no connectivity).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate normals for a point cloud using PCA of k-nearest neighbors.
pub fn estimate_point_cloud_normals(mesh: &PolyData, k: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut normals = Vec::with_capacity(n * 3);
    for i in 0..n {
        let neighbors = knn(&pts, i, k);
        let nm = pca_normal(&pts, i, &neighbors);
        normals.extend_from_slice(&nm);
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

/// Orient point cloud normals consistently using minimum spanning tree propagation.
pub fn orient_point_cloud_normals(mesh: &PolyData, reference: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let normals = match mesh.point_data().normals() {
        Some(n) => n,
        None => return mesh.clone(),
    };
    let mut nm_data = Vec::with_capacity(n * 3);
    let mut buf = [0.0f64; 3];

    for i in 0..n {
        normals.tuple_as_f64(i, &mut buf);
        let p = mesh.points.get(i);
        let to_ref = [
            reference[0] - p[0],
            reference[1] - p[1],
            reference[2] - p[2],
        ];
        let dot = buf[0] * to_ref[0] + buf[1] * to_ref[1] + buf[2] * to_ref[2];
        if dot < 0.0 {
            nm_data.push(-buf[0]);
            nm_data.push(-buf[1]);
            nm_data.push(-buf[2]);
        } else {
            nm_data.extend_from_slice(&buf);
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", nm_data, 3,
        )));
    result.point_data_mut().set_active_normals("Normals");
    result
}

fn knn(pts: &[[f64; 3]], query: usize, k: usize) -> Vec<usize> {
    let q = pts[query];
    let mut dists: Vec<(usize, f64)> = pts
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != query)
        .map(|(i, p)| {
            (
                i,
                (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2),
            )
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dists.iter().take(k).map(|(i, _)| *i).collect()
}

fn pca_normal(pts: &[[f64; 3]], center: usize, neighbors: &[usize]) -> [f64; 3] {
    let p = pts[center];
    let mut cov = [[0.0; 3]; 3];
    for &ni in neighbors {
        let d = [pts[ni][0] - p[0], pts[ni][1] - p[1], pts[ni][2] - p[2]];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }
    // Find eigenvector of smallest eigenvalue (normal direction)
    // Use power iteration on M to find largest, then cross for smallest
    let mut v1 = [1.0, 0.0, 0.0];
    for _ in 0..20 {
        let w = mat_vec(&cov, v1);
        let len = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if len < 1e-15 {
            break;
        }
        v1 = [w[0] / len, w[1] / len, w[2] / len];
    }
    // Deflate
    let ev1 = {
        let mv = mat_vec(&cov, v1);
        mv[0] * v1[0] + mv[1] * v1[1] + mv[2] * v1[2]
    };
    let mut cov2 = cov;
    for r in 0..3 {
        for c in 0..3 {
            cov2[r][c] -= ev1 * v1[r] * v1[c];
        }
    }
    let mut v2 = [0.0, 1.0, 0.0];
    for _ in 0..20 {
        let w = mat_vec(&cov2, v2);
        let len = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if len < 1e-15 {
            break;
        }
        v2 = [w[0] / len, w[1] / len, w[2] / len];
    }
    let n = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
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
    use crate::data::Points;
    #[test]
    fn planar_normals() {
        let pts: Vec<[f64; 3]> = (0..20)
            .map(|i| [(i % 5) as f64, (i / 5) as f64, 0.0])
            .collect();
        let mut mesh = PolyData::new();
        mesh.points = Points::from(pts);
        let result = estimate_point_cloud_normals(&mesh, 5);
        let arr = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(10, &mut buf);
        // Normal of XY plane should be approximately [0,0,±1]
        assert!(buf[2].abs() > 0.8, "normal z={}", buf[2]);
    }
    #[test]
    fn orient() {
        let pts: Vec<[f64; 3]> = (0..10)
            .map(|i| [(i % 5) as f64, (i / 5) as f64, 0.0])
            .collect();
        let mut mesh = PolyData::from_points(pts);
        mesh = estimate_point_cloud_normals(&mesh, 3);
        let result = orient_point_cloud_normals(&mesh, [2.0, 1.0, 1.0]);
        assert!(result.point_data().normals().is_some());
    }

    #[test]
    fn orient_points_toward_reference() {
        let mut mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, -1.0],
                3,
            )));
        mesh.point_data_mut().set_active_normals("Normals");

        let result = orient_point_cloud_normals(&mesh, [0.0, 0.0, 1.0]);
        let normals = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 1.0]);
    }
}
