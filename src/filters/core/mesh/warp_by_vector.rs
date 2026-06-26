//! Warp mesh by vector or scalar data arrays.

use crate::data::{Points, PolyData};

/// Warp mesh vertices by a vector point data array.
pub fn warp_by_vector(mesh: &PolyData, array_name: &str, scale: f64) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 && a.num_tuples() >= n => a,
        _ => return mesh.clone(),
    };
    let mut pts = Points::<f64>::new();
    let mut buf = [0.0f64; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        arr.tuple_as_f64(i, &mut buf);
        pts.push([
            p[0] + buf[0] * scale,
            p[1] + buf[1] * scale,
            p[2] + buf[2] * scale,
        ]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

/// Warp mesh vertices along their normals by a scalar array.
pub fn warp_by_scalar(mesh: &PolyData, array_name: &str, scale: f64) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return mesh.clone(),
    };
    let normals = point_normals(mesh);
    let mut pts = Points::<f64>::new();
    let mut buf = [0.0f64];
    for i in 0..n {
        let p = mesh.points.get(i);
        arr.tuple_as_f64(i, &mut buf);
        let nm = &normals[i];
        pts.push([
            p[0] + nm[0] * buf[0] * scale,
            p[1] + nm[1] * buf[0] * scale,
            p[2] + nm[2] * buf[0] * scale,
        ]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

/// Warp by a procedural displacement function.
pub fn warp_by_function(mesh: &PolyData, f: impl Fn([f64; 3]) -> [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = f(p);
        pts.push([p[0] + d[0], p[1] + d[1], p[2] + d[2]]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

fn point_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    if let Some(normals) = mesh
        .point_data()
        .normals()
        .or_else(|| mesh.point_data().get_array("Normals"))
        .filter(|a| a.num_components() == 3 && a.num_tuples() >= n)
    {
        let mut out = Vec::with_capacity(n);
        let mut buf = [0.0f64; 3];
        for i in 0..n {
            normals.tuple_as_f64(i, &mut buf);
            out.push(buf);
        }
        return out;
    }

    vec![[0.0, 0.0, 1.0]; n]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn vector_warp() {
        let mut m = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "disp",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 2.0],
                3,
            )));
        let result = warp_by_vector(&m, "disp", 1.0);
        assert!((result.points.get(0)[2] - 1.0).abs() < 0.01);
        assert!((result.points.get(1)[2] - 2.0).abs() < 0.01);
    }
    #[test]
    fn function_warp() {
        let m = PolyData::from_points(vec![[1.0, 0.0, 0.0]]);
        let result = warp_by_function(&m, |p| [0.0, 0.0, p[0]]);
        assert!((result.points.get(0)[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn scalar_warp_uses_active_normals() {
        let mut m = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vec![2.0], 1)));
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "n",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        m.point_data_mut().set_active_normals("n");

        let result = warp_by_scalar(&m, "s", 0.5);
        assert_eq!(result.points.get(0), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn scalar_warp_defaults_to_z_normal() {
        let mut m = PolyData::from_points(vec![[0.0, 0.0, 1.0]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vec![2.0], 1)));

        let result = warp_by_scalar(&m, "s", 0.5);
        assert_eq!(result.points.get(0), [0.0, 0.0, 2.0]);
    }

    #[test]
    fn short_vector_array_returns_input_clone() {
        let mut m = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "disp",
                vec![0.0, 0.0, 1.0],
                3,
            )));

        let result = warp_by_vector(&m, "disp", 1.0);
        assert_eq!(result.points.get(1), [1.0, 0.0, 0.0]);
    }
}
