//! Compute distance from each vertex to a reference point.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn distance_to_point(mesh: &PolyData, point: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - point[0]).powi(2) + (p[1] - point[1]).powi(2) + (p[2] - point[2]).powi(2))
                .sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Distance", data, 1)));
    r.point_data_mut().set_active_scalars("Distance");
    r
}
pub fn distance_to_line(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let dl =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    let d = if dl > 1e-15 {
        [direction[0] / dl, direction[1] / dl, direction[2] / dl]
    } else {
        [0.0, 0.0, 1.0]
    };
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let v = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
            let proj = v[0] * d[0] + v[1] * d[1] + v[2] * d[2];
            let perp = [v[0] - proj * d[0], v[1] - proj * d[1], v[2] - proj * d[2]];
            (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Distance", data, 1)));
    r.point_data_mut().set_active_scalars("Distance");
    r
}
pub fn distance_to_plane(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let nl = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    let nn = if nl > 1e-15 {
        [normal[0] / nl, normal[1] / nl, normal[2] / nl]
    } else {
        [0.0, 0.0, 1.0]
    };
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - origin[0]) * nn[0] + (p[1] - origin[1]) * nn[1] + (p[2] - origin[2]) * nn[2])
                .abs()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Distance", data, 1)));
    r.point_data_mut().set_active_scalars("Distance");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pt() {
        let m = PolyData::from_triangles(
            vec![[3.0, 4.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_point(&m, [0.0, 0.0, 0.0]);
        let mut buf = [0.0];
        r.point_data()
            .get_array("Distance")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_plane() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_plane(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let mut buf = [0.0];
        r.point_data()
            .get_array("Distance")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
}
