//! Feature line field (ridge/valley direction field on mesh).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn ridge_direction_field(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let nm = calc_nm(mesh);
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
    // For each vertex, find direction of minimum scalar variation (ridge direction)
    let mut dirs = Vec::with_capacity(n * 3);
    for i in 0..n {
        if nb[i].len() < 2 {
            dirs.extend_from_slice(&[0.0, 0.0, 0.0]);
            continue;
        }
        let p = mesh.points.get(i);
        let ni = nm[i];
        let vi = vals[i];
        let mut min_var = f64::INFINITY;
        let mut min_dir = [1.0, 0.0, 0.0];
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let proj = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
            let tangent = [
                d[0] - proj * ni[0],
                d[1] - proj * ni[1],
                d[2] - proj * ni[2],
            ];
            let tl = (tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2])
                .sqrt();
            if tl < 1e-15 {
                continue;
            }
            let dv = (vals[j] - vi).abs() / tl;
            if dv < min_var {
                min_var = dv;
                min_dir = [tangent[0] / tl, tangent[1] / tl, tangent[2] / tl];
            }
        }
        dirs.extend_from_slice(&min_dir);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RidgeDir", dirs, 3)));
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
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 2.0, 1.5],
                1,
            )));
        let r = ridge_direction_field(&m, "h");
        assert!(r.point_data().get_array("RidgeDir").is_some());
        assert_eq!(
            r.point_data()
                .get_array("RidgeDir")
                .unwrap()
                .num_components(),
            3
        );
    }
}
