//! Define and manipulate tangent vector fields on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn radial_vector_field(mesh: &PolyData, center: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        // Project onto tangent plane
        let ni = nm[i];
        let dot = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
        let t = [d[0] - dot * ni[0], d[1] - dot * ni[1], d[2] - dot * ni[2]];
        let tl = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-15);
        data.push(t[0] / tl);
        data.push(t[1] / tl);
        data.push(t[2] / tl);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VectorField",
            data,
            3,
        )));
    r
}
pub fn rotational_vector_field(mesh: &PolyData, axis: [f64; 3], center: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let al = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2])
        .sqrt()
        .max(1e-15);
    let ax = [axis[0] / al, axis[1] / al, axis[2] / al];
    let nm = calc_nm(mesh);
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        let cross = [
            ax[1] * d[2] - ax[2] * d[1],
            ax[2] * d[0] - ax[0] * d[2],
            ax[0] * d[1] - ax[1] * d[0],
        ];
        let ni = nm[i];
        let dot = cross[0] * ni[0] + cross[1] * ni[1] + cross[2] * ni[2];
        let t = [
            cross[0] - dot * ni[0],
            cross[1] - dot * ni[1],
            cross[2] - dot * ni[2],
        ];
        let tl = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-15);
        data.push(t[0] / tl);
        data.push(t[1] / tl);
        data.push(t[2] / tl);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VectorField",
            data,
            3,
        )));
    r
}
pub fn gradient_vector_field(mesh: &PolyData, scalar_array: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(scalar_array) {
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
    let mut grad = vec![[0.0f64; 3]; n];
    let mut wt = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let nm = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let a2 = nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2];
        if a2 < 1e-30 {
            continue;
        }
        let df1 = vals[ids[1]] - vals[ids[0]];
        let df2 = vals[ids[2]] - vals[ids[0]];
        let pe2 = [
            (nm[1] * e2[2] - nm[2] * e2[1]) / a2,
            (nm[2] * e2[0] - nm[0] * e2[2]) / a2,
            (nm[0] * e2[1] - nm[1] * e2[0]) / a2,
        ];
        let pe1 = [
            (nm[1] * e1[2] - nm[2] * e1[1]) / a2,
            (nm[2] * e1[0] - nm[0] * e1[2]) / a2,
            (nm[0] * e1[1] - nm[1] * e1[0]) / a2,
        ];
        let g = [
            df1 * pe2[0] - df2 * pe1[0],
            df1 * pe2[1] - df2 * pe1[1],
            df1 * pe2[2] - df2 * pe1[2],
        ];
        for &vi in &ids {
            grad[vi][0] += g[0];
            grad[vi][1] += g[1];
            grad[vi][2] += g[2];
            wt[vi] += 1.0;
        }
    }
    for i in 0..n {
        if wt[i] > 0.0 {
            grad[i][0] /= wt[i];
            grad[i][1] /= wt[i];
            grad[i][2] /= wt[i];
        }
    }
    let data: Vec<f64> = grad.iter().flat_map(|g| g.iter().copied()).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VectorField",
            data,
            3,
        )));
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
    fn test_radial() {
        let m = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [2, 3, 0]],
        );
        let r = radial_vector_field(&m, [0.0, 0.0, 0.0]);
        assert!(r.point_data().get_array("VectorField").is_some());
        assert_eq!(
            r.point_data()
                .get_array("VectorField")
                .unwrap()
                .num_components(),
            3
        );
    }
    #[test]
    fn test_gradient() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 2.0, 1.0],
                1,
            )));
        let r = gradient_vector_field(&m, "h");
        assert!(r.point_data().get_array("VectorField").is_some());
    }
}
