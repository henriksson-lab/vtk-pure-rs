//! Compute signed distance from each vertex to a plane.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn signed_distance_to_plane(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let nl = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
        .sqrt()
        .max(1e-15);
    let nn = [normal[0] / nl, normal[1] / nl, normal[2] / nl];
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            (p[0] - origin[0]) * nn[0] + (p[1] - origin[1]) * nn[1] + (p[2] - origin[2]) * nn[2]
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SignedDistance",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("SignedDistance");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, -3.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = signed_distance_to_plane(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let mut buf = [0.0];
        r.point_data()
            .get_array("SignedDistance")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
        r.point_data()
            .get_array("SignedDistance")
            .unwrap()
            .tuple_as_f64(1, &mut buf);
        assert!((buf[0] + 3.0).abs() < 1e-10);
    }
}
