//! Full shape index classification from principal curvatures.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn full_shape_index(mesh: &PolyData) -> PolyData {
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
    let mut si_data = Vec::with_capacity(n);
    let mut curv_type = Vec::with_capacity(n);
    for i in 0..n {
        if nb[i].is_empty() {
            si_data.push(0.0);
            curv_type.push(0.0);
            continue;
        }
        let p = mesh.points.get(i);
        let ni = nm[i];
        let mut curvatures = Vec::new();
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
            let proj = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
            curvatures.push(2.0 * proj / (dist * dist));
        }
        if curvatures.is_empty() {
            si_data.push(0.0);
            curv_type.push(0.0);
            continue;
        }
        curvatures.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let k1 = curvatures[curvatures.len() - 1];
        let k2 = curvatures[0];
        // Shape index = 2/pi * atan((k1+k2)/(k1-k2))
        let si = if (k1 - k2).abs() > 1e-15 {
            (2.0 / std::f64::consts::PI) * ((k1 + k2) / (k1 - k2)).atan()
        } else {
            0.0
        };
        si_data.push(si);
        // Classification: -1=cup, -0.5=rut, 0=saddle, 0.5=ridge, 1=cap
        let ct = if si < -0.75 {
            -1.0
        } else if si < -0.25 {
            -0.5
        } else if si < 0.25 {
            0.0
        } else if si < 0.75 {
            0.5
        } else {
            1.0
        };
        curv_type.push(ct);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeIndex",
            si_data,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureClass",
            curv_type,
            1,
        )));
    r.point_data_mut().set_active_scalars("ShapeIndex");
    r
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
        let r = full_shape_index(&m);
        assert!(r.point_data().get_array("ShapeIndex").is_some());
        assert!(r.point_data().get_array("CurvatureClass").is_some());
    }
}
