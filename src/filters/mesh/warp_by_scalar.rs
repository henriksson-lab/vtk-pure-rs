//! Warp mesh geometry along normals by a scalar field.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Warp mesh points along their normals scaled by a scalar array.
pub fn warp_by_scalar(mesh: &PolyData, scalar_name: &str, scale: f64) -> PolyData {
    let n = mesh.points.len();
    let scalars = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return mesh.clone(),
    };
    let normals = mesh
        .point_data()
        .normals()
        .or_else(|| mesh.point_data().get_array("Normals"))
        .filter(|a| a.num_components() == 3 && a.num_tuples() >= n);

    let mut sbuf = [0.0f64];
    let mut nbuf = [0.0f64; 3];
    let mut result = mesh.clone();
    for i in 0..n {
        scalars.tuple_as_f64(i, &mut sbuf);
        if let Some(normals) = normals {
            normals.tuple_as_f64(i, &mut nbuf);
        } else {
            nbuf = [0.0, 0.0, 1.0];
        }
        let p = mesh.points.get(i);
        let d = sbuf[0] * scale;
        result.points.set(
            i,
            [p[0] + d * nbuf[0], p[1] + d * nbuf[1], p[2] + d * nbuf[2]],
        );
    }
    result
}

/// Warp mesh points along Z axis by a scalar array.
pub fn warp_z_by_scalar(mesh: &PolyData, scalar_name: &str, scale: f64) -> PolyData {
    let n = mesh.points.len();
    let scalars = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let mut result = mesh.clone();
    for i in 0..n {
        scalars.tuple_as_f64(i, &mut buf);
        let p = mesh.points.get(i);
        result.points.set(i, [p[0], p[1], p[2] + buf[0] * scale]);
    }
    result
}

/// Warp mesh points radially from center by a scalar array.
pub fn warp_radial_by_scalar(
    mesh: &PolyData,
    scalar_name: &str,
    center: [f64; 3],
    scale: f64,
) -> PolyData {
    let n = mesh.points.len();
    let scalars = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let mut result = mesh.clone();
    for i in 0..n {
        scalars.tuple_as_f64(i, &mut buf);
        let p = mesh.points.get(i);
        let dir = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if len < 1e-15 {
            continue;
        }
        let d = buf[0] * scale;
        result.points.set(
            i,
            [
                p[0] + d * dir[0] / len,
                p[1] + d * dir[1] / len,
                p[2] + d * dir[2] / len,
            ],
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_warp_z() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let r = warp_z_by_scalar(&mesh, "h", 0.5);
        let p = r.points.get(2);
        assert!((p[2] - 1.5).abs() < 1e-10);
    }
    #[test]
    fn test_warp_normal() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![1.0, 1.0, 1.0],
                1,
            )));
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));
        let r = warp_by_scalar(&mesh, "h", 2.0);
        let p = r.points.get(0);
        assert!((p[2] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_warp_radial() {
        let mut mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 1.0, 1.0],
                1,
            )));
        let r = warp_radial_by_scalar(&mesh, "s", [0.0, 0.0, 0.0], 0.5);
        let p = r.points.get(0);
        assert!((p[0] - 1.5).abs() < 1e-10); // moved outward
    }

    #[test]
    fn warp_scalar_defaults_to_z_normal() {
        let mut mesh = PolyData::from_points(vec![[0.0, 0.0, 1.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("h", vec![2.0], 1)));

        let r = warp_by_scalar(&mesh, "h", 0.5);
        assert_eq!(r.points.get(0), [0.0, 0.0, 2.0]);
    }

    #[test]
    fn short_scalar_array_returns_input_clone() {
        let mut mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("h", vec![1.0], 1)));

        let r = warp_z_by_scalar(&mesh, "h", 1.0);
        assert_eq!(r.points.get(1), [1.0, 0.0, 0.0]);
    }
}
