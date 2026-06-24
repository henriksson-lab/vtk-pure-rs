//! Attach position coordinates as separate point data arrays.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn attach_position_arrays(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let x: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[0]).collect();
    let y: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[1]).collect();
    let z: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[2]).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("X", x, 1)));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Y", y, 1)));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Z", z, 1)));
    r
}
pub fn attach_radius_array(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Radius", data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pos() {
        let m = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_position_arrays(&m);
        assert!(r.point_data().get_array("X").is_some());
        let mut buf = [0.0];
        r.point_data()
            .get_array("X")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_rad() {
        let m = PolyData::from_triangles(
            vec![[3.0, 4.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_radius_array(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("Radius")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
}
