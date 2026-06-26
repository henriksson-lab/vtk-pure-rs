use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate normals for a point cloud using PCA of local neighborhoods.
///
/// For each point, finds `k_neighbors` nearest neighbors (by Euclidean
/// distance), computes the 3x3 covariance matrix of the neighborhood,
/// and takes the eigenvector corresponding to the smallest eigenvalue
/// as the estimated normal direction. The normals are added as a
/// 3-component "Normals" point data array.
pub fn estimate_point_cloud_normals(input: &PolyData, k_neighbors: usize) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 || k_neighbors == 0 {
        return input.clone();
    }

    let k: usize = k_neighbors.min(n);
    let mut normal_data: Vec<f64> = Vec::with_capacity(n * 3);

    for i in 0..n {
        let pi = input.points.get(i);

        // Find k nearest neighbors by brute force
        let mut dists: Vec<(f64, usize)> = Vec::with_capacity(n);
        for j in 0..n {
            let pj = input.points.get(j);
            let dx: f64 = pj[0] - pi[0];
            let dy: f64 = pj[1] - pi[1];
            let dz: f64 = pj[2] - pi[2];
            let d2: f64 = dx * dx + dy * dy + dz * dz;
            dists.push((d2, j));
        }
        dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Compute centroid of k nearest neighbors
        let mut cx: f64 = 0.0;
        let mut cy: f64 = 0.0;
        let mut cz: f64 = 0.0;
        for idx in 0..k {
            let pj = input.points.get(dists[idx].1);
            cx += pj[0];
            cy += pj[1];
            cz += pj[2];
        }
        let k_f: f64 = k as f64;
        cx /= k_f;
        cy /= k_f;
        cz /= k_f;

        // Compute 3x3 covariance matrix (symmetric)
        let mut cov: [f64; 9] = [0.0; 9]; // row-major 3x3
        for idx in 0..k {
            let pj = input.points.get(dists[idx].1);
            let dx: f64 = pj[0] - cx;
            let dy: f64 = pj[1] - cy;
            let dz: f64 = pj[2] - cz;
            cov[0] += dx * dx;
            cov[1] += dx * dy;
            cov[2] += dx * dz;
            cov[3] += dy * dx;
            cov[4] += dy * dy;
            cov[5] += dy * dz;
            cov[6] += dz * dx;
            cov[7] += dz * dy;
            cov[8] += dz * dz;
        }

        // Find smallest eigenvector via power iteration on the shifted matrix
        // We find the largest eigenvector of (trace(C)*I - C) which corresponds
        // to the smallest eigenvector of C.
        let normal = smallest_eigenvector(&cov);
        normal_data.push(normal[0]);
        normal_data.push(normal[1]);
        normal_data.push(normal[2]);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals",
            normal_data,
            3,
        )));
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

/// Find the eigenvector corresponding to the smallest eigenvalue of a 3x3
/// symmetric positive semi-definite matrix using the analytical method.
fn smallest_eigenvector(cov: &[f64; 9]) -> [f64; 3] {
    // Use Jacobi eigenvalue algorithm for 3x3 symmetric matrix
    let mut a: [f64; 9] = *cov;
    // Eigenvectors as columns of v
    let mut v: [f64; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    // Jacobi iterations
    for _ in 0..50 {
        // Find largest off-diagonal element
        let mut max_val: f64 = 0.0;
        let mut p: usize = 0;
        let mut q: usize = 1;
        for ii in 0..3 {
            for jj in (ii + 1)..3 {
                let val: f64 = a[ii * 3 + jj].abs();
                if val > max_val {
                    max_val = val;
                    p = ii;
                    q = jj;
                }
            }
        }
        if max_val < 1e-15 {
            break;
        }

        // Compute rotation
        let app: f64 = a[p * 3 + p];
        let aqq: f64 = a[q * 3 + q];
        let apq: f64 = a[p * 3 + q];

        let theta: f64 = if (app - aqq).abs() < 1e-20 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };

        let c: f64 = theta.cos();
        let s: f64 = theta.sin();

        // Apply Givens rotation to a
        let mut new_a: [f64; 9] = a;
        new_a[p * 3 + p] = c * c * app + 2.0 * c * s * apq + s * s * aqq;
        new_a[q * 3 + q] = s * s * app - 2.0 * c * s * apq + c * c * aqq;
        new_a[p * 3 + q] = 0.0;
        new_a[q * 3 + p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp: f64 = a[r * 3 + p];
                let arq: f64 = a[r * 3 + q];
                new_a[r * 3 + p] = c * arp + s * arq;
                new_a[p * 3 + r] = c * arp + s * arq;
                new_a[r * 3 + q] = -s * arp + c * arq;
                new_a[q * 3 + r] = -s * arp + c * arq;
            }
        }
        a = new_a;

        // Apply rotation to eigenvectors
        for r in 0..3 {
            let vp: f64 = v[r * 3 + p];
            let vq: f64 = v[r * 3 + q];
            v[r * 3 + p] = c * vp + s * vq;
            v[r * 3 + q] = -s * vp + c * vq;
        }
    }

    // Find index of smallest eigenvalue
    let mut min_idx: usize = 0;
    let mut min_val: f64 = a[0];
    for i in 1..3 {
        if a[i * 3 + i] < min_val {
            min_val = a[i * 3 + i];
            min_idx = i;
        }
    }

    // Extract corresponding eigenvector (column min_idx of v)
    let mut normal: [f64; 3] = [v[0 * 3 + min_idx], v[1 * 3 + min_idx], v[2 * 3 + min_idx]];
    let len: f64 = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len > 1e-20 {
        normal[0] /= len;
        normal[1] /= len;
        normal[2] /= len;
    }

    normal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planar_points_normal_is_z() {
        // Points in the XY plane should have normals along Z
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]);

        let result = estimate_point_cloud_normals(&pd, 5);
        let normals = result.point_data().get_array("Normals").unwrap();
        assert!(result.point_data().normals().is_some());
        assert_eq!(normals.num_components(), 3);
        assert_eq!(normals.num_tuples(), 5);

        let mut buf = [0.0f64; 3];
        for i in 0..5 {
            normals.tuple_as_f64(i, &mut buf);
            // Normal should be (0,0,+/-1)
            assert!(buf[2].abs() > 0.99, "normal z={} at point {}", buf[2], i);
            assert!(buf[0].abs() < 0.01, "normal x={} at point {}", buf[0], i);
            assert!(buf[1].abs() < 0.01, "normal y={} at point {}", buf[1], i);
        }
    }

    #[test]
    fn empty_point_cloud() {
        let pd = PolyData::new();
        let result = estimate_point_cloud_normals(&pd, 5);
        assert!(result.point_data().get_array("Normals").is_none());
    }

    #[test]
    fn single_point_returns_some_normal() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = estimate_point_cloud_normals(&pd, 1);
        let normals = result.point_data().get_array("Normals").unwrap();
        assert_eq!(normals.num_tuples(), 1);
        // With a single point the covariance is zero, so we just check it doesn't crash
        let mut buf = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut buf);
        let len: f64 = (buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2]).sqrt();
        // len could be 0 or 1, either is acceptable
        assert!(len < 1.01);
    }
}
