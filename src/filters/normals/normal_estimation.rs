use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Estimate normals for an unstructured point cloud.
///
/// Uses PCA (principal component analysis) on k-nearest-neighbor
/// neighborhoods to estimate a normal direction at each point.
/// The normals are oriented toward the origin, matching VTK's default
/// orientation-point mode.
///
/// Adds a "PCANormals" array to point data.
pub fn normal_estimation(input: &PolyData, k: usize) -> PolyData {
    let n = input.points.len();
    let k = k.max(1).min(n.max(1));

    if n == 0 {
        return input.clone();
    }

    // Build k-d tree
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut normals_arr = vec![[0.0f64; 3]; n];

    // Estimate normal at each point via PCA on k-NN
    for i in 0..n {
        let neighbors = tree.k_nearest(pts[i], k);

        // Compute centroid of neighborhood
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let count = neighbors.len() as f64;
        for &(idx, _) in &neighbors {
            cx += pts[idx][0];
            cy += pts[idx][1];
            cz += pts[idx][2];
        }
        cx /= count;
        cy /= count;
        cz /= count;

        // Build covariance matrix
        let mut cov = [[0.0f64; 3]; 3];
        for &(idx, _) in &neighbors {
            let dx = pts[idx][0] - cx;
            let dy = pts[idx][1] - cy;
            let dz = pts[idx][2] - cz;
            cov[0][0] += dx * dx;
            cov[0][1] += dx * dy;
            cov[0][2] += dx * dz;
            cov[1][1] += dy * dy;
            cov[1][2] += dy * dz;
            cov[2][2] += dz * dz;
        }
        for row in &mut cov {
            for value in row {
                *value /= count;
            }
        }
        cov[1][0] = cov[0][1];
        cov[2][0] = cov[0][2];
        cov[2][1] = cov[1][2];

        let normal = smallest_eigenvector(&cov);
        normals_arr[i] = normal;
    }

    for (point, normal) in pts.iter().zip(normals_arr.iter_mut()) {
        let to_origin = [-point[0], -point[1], -point[2]];
        if dot(to_origin, *normal) < 0.0 {
            *normal = negate(*normal);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    *pd.point_data_mut() = input.point_data().clone();
    *pd.field_data_mut() = input.field_data().clone();
    let flat: Vec<f64> = normals_arr.iter().flat_map(|n| n.iter().copied()).collect();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PCANormals",
            flat,
            3,
        )));
    pd.point_data_mut().set_active_normals("PCANormals");
    pd
}

/// Find the eigenvector corresponding to the smallest eigenvalue
/// of a 3x3 symmetric matrix using Jacobi rotations.
fn smallest_eigenvector(m: &[[f64; 3]; 3]) -> [f64; 3] {
    let mut a = *m;
    let mut v = [[0.0f64; 3]; 3];
    for i in 0..3 {
        v[i][i] = 1.0;
    }

    for _ in 0..50 {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max = a[0][1].abs();
        for i in 0..3 {
            for j in (i + 1)..3 {
                if a[i][j].abs() > max {
                    max = a[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }
        if max < 1e-15 {
            break;
        }

        let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                a[r][p] = c * arp - s * arq;
                a[p][r] = a[r][p];
                a[r][q] = s * arp + c * arq;
                a[q][r] = a[r][q];
            }
            let vrp = v[r][p];
            let vrq = v[r][q];
            v[r][p] = c * vrp - s * vrq;
            v[r][q] = s * vrp + c * vrq;
        }
    }

    let min_col = if a[0][0] <= a[1][1] && a[0][0] <= a[2][2] {
        0
    } else if a[1][1] <= a[2][2] {
        1
    } else {
        2
    };
    let mut normal = [v[0][min_col], v[1][min_col], v[2][min_col]];
    let len = dot(normal, normal).sqrt();
    if len > 1e-15 {
        normal[0] /= len;
        normal[1] /= len;
        normal[2] /= len;
    }
    normal
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn negate(v: [f64; 3]) -> [f64; 3] {
    [-v[0], -v[1], -v[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_plane_normals() {
        let mut pd = PolyData::new();
        // Grid of points in XY plane
        for j in 0..5 {
            for i in 0..5 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }

        let result = normal_estimation(&pd, 6);
        let arr = result.point_data().get_array("PCANormals").unwrap();

        // All normals should be approximately [0, 0, ±1]
        let mut buf = [0.0f64; 3];
        for i in 0..25 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[2].abs() > 0.9, "z-normal at {} = {}", i, buf[2]);
        }
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = normal_estimation(&pd, 3);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn sphere_points() {
        use std::f64::consts::PI;
        let mut pd = PolyData::new();
        // Points on a unit sphere
        for i in 0..20 {
            let phi = PI * i as f64 / 19.0;
            for j in 0..20 {
                let theta = 2.0 * PI * j as f64 / 20.0;
                pd.points
                    .push([phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos()]);
            }
        }

        let result = normal_estimation(&pd, 8);
        let arr = result.point_data().get_array("PCANormals").unwrap();

        // Normals should roughly point radially
        let mut buf = [0.0f64; 3];
        let mut dot_sum = 0.0;
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            arr.tuple_as_f64(i, &mut buf);
            // dot(normal, position) should be close to ±1 for a unit sphere
            let d = (p[0] * buf[0] + p[1] * buf[1] + p[2] * buf[2]).abs();
            dot_sum += d;
        }
        let avg_dot = dot_sum / pd.points.len() as f64;
        assert!(avg_dot > 0.8, "avg radial alignment = {}", avg_dot);
    }
}
