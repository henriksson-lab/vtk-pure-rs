use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Estimate normals for an unstructured point cloud.
///
/// Uses PCA (principal component analysis) on k-nearest-neighbor
/// neighborhoods to estimate a normal direction at each point.
/// The normals are oriented consistently by propagating from an
/// initial point using a minimum spanning tree heuristic.
///
/// Adds a "Normals" array to point data.
pub fn normal_estimation(input: &PolyData, k: usize) -> PolyData {
    let n = input.points.len();
    let k = k.max(3);

    if n < 3 {
        return input.clone();
    }

    // Build k-d tree
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut normals_arr = vec![[0.0f64; 3]; n];

    // Estimate normal at each point via PCA on k-NN
    for i in 0..n {
        let neighbors = tree.k_nearest(pts[i], k + 1); // +1 because self is included

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
        cov[1][0] = cov[0][1];
        cov[2][0] = cov[0][2];
        cov[2][1] = cov[1][2];

        // Find smallest eigenvector via power iteration on inverse
        // (normal is the eigenvector of smallest eigenvalue)
        // Use simple approach: the normal is approximately the cross product
        // of two vectors in the tangent plane found by SVD-like iteration
        let normal = smallest_eigenvector(&cov);
        normals_arr[i] = normal;
    }

    // Orient normals consistently: start from point with max Z,
    // orient it upward, then propagate via nearest neighbors
    let mut oriented = vec![false; n];
    let start = (0..n)
        .max_by(|&a, &b| pts[a][2].partial_cmp(&pts[b][2]).unwrap())
        .unwrap_or(0);

    // Orient start normal to point "up" (positive Z)
    if normals_arr[start][2] < 0.0 {
        normals_arr[start] = negate(normals_arr[start]);
    }
    oriented[start] = true;

    // BFS propagation
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);

    while let Some(curr) = queue.pop_front() {
        let neighbors = tree.k_nearest(pts[curr], k + 1);
        for &(idx, _) in &neighbors {
            if oriented[idx] {
                continue;
            }
            // Orient to agree with current point's normal
            if dot(normals_arr[idx], normals_arr[curr]) < 0.0 {
                normals_arr[idx] = negate(normals_arr[idx]);
            }
            oriented[idx] = true;
            queue.push_back(idx);
        }
    }

    let mut pd = input.clone();
    let flat: Vec<f64> = normals_arr.iter().flat_map(|n| n.iter().copied()).collect();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", flat, 3)));
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

/// Find the eigenvector corresponding to the smallest eigenvalue
/// of a 3×3 symmetric matrix using the power method on the adjugate.
fn smallest_eigenvector(m: &[[f64; 3]; 3]) -> [f64; 3] {
    // Use two iterations of inverse power method with shift
    // For a simpler approach: compute cross products of rows
    // and pick the one with largest magnitude
    let r0 = [m[0][0], m[0][1], m[0][2]];
    let r1 = [m[1][0], m[1][1], m[1][2]];
    let r2 = [m[2][0], m[2][1], m[2][2]];

    let c01 = cross(r0, r1);
    let c02 = cross(r0, r2);
    let c12 = cross(r1, r2);

    let l01 = dot(c01, c01);
    let l02 = dot(c02, c02);
    let l12 = dot(c12, c12);

    let best = if l01 >= l02 && l01 >= l12 {
        c01
    } else if l02 >= l12 {
        c02
    } else {
        c12
    };

    let len = dot(best, best).sqrt();
    if len > 1e-15 {
        [best[0] / len, best[1] / len, best[2] / len]
    } else {
        [0.0, 0.0, 1.0]
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
        let arr = result.point_data().get_array("Normals").unwrap();

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
        let arr = result.point_data().get_array("Normals").unwrap();

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
