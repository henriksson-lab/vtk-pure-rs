//! Compute signed distance from each vertex to an arbitrary plane.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn distance_to_plane(mesh: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let nl = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    let nn = if nl > 1e-15 {
        [normal[0] / nl, normal[1] / nl, normal[2] / nl]
    } else {
        [0.0, 0.0, 1.0]
    };
    let d = -(nn[0] * point[0] + nn[1] * point[1] + nn[2] * point[2]);
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            nn[0] * p[0] + nn[1] * p[1] + nn[2] * p[2] + d
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PlaneDistance",
            dists,
            1,
        )));
    result.point_data_mut().set_active_scalars("PlaneDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_plane_dist() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_plane(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let arr = r.point_data().get_array("PlaneDistance").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0);
        arr.tuple_as_f64(2, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-9);
    }
}
