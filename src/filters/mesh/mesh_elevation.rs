//! Compute elevation (Z coordinate) as scalar array.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn elevation_z(mesh: &PolyData) -> PolyData {
    elevation_axis(mesh, 2)
}
pub fn elevation_x(mesh: &PolyData) -> PolyData {
    elevation_axis(mesh, 0)
}
pub fn elevation_y(mesh: &PolyData) -> PolyData {
    elevation_axis(mesh, 1)
}
pub fn elevation_axis(mesh: &PolyData, axis: usize) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[axis]).collect();
    let name = match axis {
        0 => "ElevationX",
        1 => "ElevationY",
        _ => "ElevationZ",
    };
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)));
    r.point_data_mut().set_active_scalars(name);
    r
}
pub fn elevation_along(mesh: &PolyData, direction: [f64; 3]) -> PolyData {
    let l =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt()
            .max(1e-15);
    let d = [direction[0] / l, direction[1] / l, direction[2] / l];
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            p[0] * d[0] + p[1] * d[1] + p[2] * d[2]
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Elevation", data, 1)));
    r.point_data_mut().set_active_scalars("Elevation");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_z() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.5, 1.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let r = elevation_z(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("ElevationZ")
            .unwrap()
            .tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }
    #[test]
    fn test_along() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = elevation_along(&m, [1.0, 1.0, 0.0]);
        assert!(r.point_data().get_array("Elevation").is_some());
    }
}
