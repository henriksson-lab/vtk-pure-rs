//! Compute gradient of a scalar array on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn scalar_gradient_on_surface(mesh: &PolyData, array_name: &str) -> PolyData {
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
        // Gradient = sum of (value * rotated_edge) / (2*area)
        let dv = [vals[ids[1]] - vals[ids[0]], vals[ids[2]] - vals[ids[0]]];
        let g = [
            dv[0] * e2[0] - dv[1] * e1[0],
            dv[0] * e2[1] - dv[1] * e1[1],
            dv[0] * e2[2] - dv[1] * e1[2],
        ];
        // Project onto triangle plane
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
        .add_array(AnyDataArray::F64(DataArray::from_vec("Gradient", data, 3)));
    r
}
pub fn scalar_gradient_magnitude(mesh: &PolyData, array_name: &str) -> PolyData {
    let r = scalar_gradient_on_surface(mesh, array_name);
    let arr = match r.point_data().get_array("Gradient") {
        Some(a) => a,
        None => return r,
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let mag: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2]).sqrt()
        })
        .collect();
    let mut result = r;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradMagnitude",
            mag,
            1,
        )));
    result.point_data_mut().set_active_scalars("GradMagnitude");
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 2.0, 1.0],
                1,
            )));
        let r = scalar_gradient_on_surface(&m, "s");
        assert!(r.point_data().get_array("Gradient").is_some());
    }
    #[test]
    fn test_mag() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = scalar_gradient_magnitude(&m, "s");
        assert!(r.point_data().get_array("GradMagnitude").is_some());
    }
}
