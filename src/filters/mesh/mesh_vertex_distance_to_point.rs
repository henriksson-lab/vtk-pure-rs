//! Distance from each vertex to a given point.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn distance_to_point(mesh: &PolyData, target: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - target[0]).powi(2) + (p[1] - target[1]).powi(2) + (p[2] - target[2]).powi(2))
                .sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PointDistance",
            dists,
            1,
        )));
    result.point_data_mut().set_active_scalars("PointDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dist_point() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_point(&mesh, [0.0, 0.0, 0.0]);
        let arr = r.point_data().get_array("PointDistance").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0);
        arr.tuple_as_f64(1, &mut b);
        assert_eq!(b[0], 3.0);
        arr.tuple_as_f64(2, &mut b);
        assert_eq!(b[0], 4.0);
    }
}
