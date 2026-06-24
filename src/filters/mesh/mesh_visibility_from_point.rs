//! Compute visibility of each face from a viewpoint.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_visibility(mesh: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let mut data = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            data.push(0.0);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nl < 1e-15 {
            data.push(0.0);
            continue;
        }
        let cx = (a[0] + b[0] + c[0]) / 3.0;
        let cy = (a[1] + b[1] + c[1]) / 3.0;
        let cz = (a[2] + b[2] + c[2]) / 3.0;
        let to_view = [viewpoint[0] - cx, viewpoint[1] - cy, viewpoint[2] - cz];
        let dot = (n[0] * to_view[0] + n[1] * to_view[1] + n[2] * to_view[2]) / nl;
        data.push(if dot > 0.0 { 1.0 } else { 0.0 });
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Visible", data, 1)));
    r
}
pub fn vertex_visibility(mesh: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let to = [
                viewpoint[0] - p[0],
                viewpoint[1] - p[1],
                viewpoint[2] - p[2],
            ];
            let dot = nm[i][0] * to[0] + nm[i][1] * to[1] + nm[i][2] * to[2];
            if dot > 0.0 {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Visible", data, 1)));
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
    fn test_face() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_visibility(&m, [0.5, 0.5, 10.0]);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Visible")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn test_vertex() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = vertex_visibility(&m, [0.5, 0.5, 10.0]);
        assert!(r.point_data().get_array("Visible").is_some());
    }
}
