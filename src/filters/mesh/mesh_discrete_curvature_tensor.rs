//! Compute full discrete curvature tensor per vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn curvature_tensor_full(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let nm = calc_nm(mesh);
    // For each vertex, estimate curvature tensor in local tangent frame
    let mut k1_data = Vec::with_capacity(n);
    let mut k2_data = Vec::with_capacity(n);
    let mut dir1_data = Vec::with_capacity(n * 3);
    let mut dir2_data = Vec::with_capacity(n * 3);
    for i in 0..n {
        if nb[i].len() < 2 {
            k1_data.push(0.0);
            k2_data.push(0.0);
            dir1_data.extend_from_slice(&[1.0, 0.0, 0.0]);
            dir2_data.extend_from_slice(&[0.0, 1.0, 0.0]);
            continue;
        }
        let p = mesh.points.get(i);
        let ni = nm[i];
        // Build local tangent frame
        let up = if ni[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let e1 = normalize(cross(ni, up));
        let e2 = cross(ni, e1);
        // Estimate curvature from neighbors
        let mut curvatures: Vec<(f64, f64, f64)> = Vec::new(); // (tangent_angle, curvature)
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if dist < 1e-15 {
                continue;
            }
            let proj = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
            let kappa = 2.0 * proj / (dist * dist);
            let tx = d[0] - proj * ni[0];
            let ty = d[1] - proj * ni[1];
            let tz = d[2] - proj * ni[2];
            let angle =
                (tx * e2[0] + ty * e2[1] + tz * e2[2]).atan2(tx * e1[0] + ty * e1[1] + tz * e1[2]);
            curvatures.push((angle.cos(), angle.sin(), kappa));
        }
        // Fit curvature tensor: kappa(theta) = k1*cos^2(theta-phi) + k2*sin^2(theta-phi)
        // Simplified: use min/max curvature
        let (mut min_k, mut max_k) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut min_dir, mut max_dir) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        for &(c, s, k) in &curvatures {
            if k < min_k {
                min_k = k;
                min_dir = [
                    c * e1[0] + s * e2[0],
                    c * e1[1] + s * e2[1],
                    c * e1[2] + s * e2[2],
                ];
            }
            if k > max_k {
                max_k = k;
                max_dir = [
                    c * e1[0] + s * e2[0],
                    c * e1[1] + s * e2[1],
                    c * e1[2] + s * e2[2],
                ];
            }
        }
        k1_data.push(max_k.min(1e6));
        k2_data.push(min_k.max(-1e6));
        dir1_data.extend_from_slice(&normalize(max_dir));
        dir2_data.extend_from_slice(&normalize(min_dir));
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Kmax", k1_data, 1)));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Kmin", k2_data, 1)));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DirMax", dir1_data, 3,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DirMin", dir2_data, 3,
        )));
    r
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = curvature_tensor_full(&m);
        assert!(r.point_data().get_array("Kmax").is_some());
        assert!(r.point_data().get_array("Kmin").is_some());
        assert!(r.point_data().get_array("DirMax").is_some());
        assert_eq!(
            r.point_data().get_array("DirMax").unwrap().num_components(),
            3
        );
    }
}
