//! Compute distance from each vertex to an arbitrary axis line.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn distance_to_axis(mesh: &PolyData, point: [f64; 3], direction: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let dl =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    let d = if dl > 1e-15 {
        [direction[0] / dl, direction[1] / dl, direction[2] / dl]
    } else {
        [0.0, 0.0, 1.0]
    };
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let v = [p[0] - point[0], p[1] - point[1], p[2] - point[2]];
            let proj = v[0] * d[0] + v[1] * d[1] + v[2] * d[2];
            let perp = [v[0] - proj * d[0], v[1] - proj * d[1], v[2] - proj * d[2]];
            (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AxisDistance",
            dists,
            1,
        )));
    result.point_data_mut().set_active_scalars("AxisDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_axis_dist() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_axis(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let arr = r.point_data().get_array("AxisDistance").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-9); // (1,0,0) is distance 1 from Z axis
    }
}
