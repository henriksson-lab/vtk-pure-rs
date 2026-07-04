//! Compute elevation scalars along a line.
use crate::data::{AnyDataArray, DataArray, DataSet, PolyData};

pub fn elevation_z(mesh: &PolyData) -> PolyData {
    let bounds = mesh.bounds();
    elevation(mesh, [0.0, 0.0, bounds.z_min], [0.0, 0.0, bounds.z_max])
}

pub fn elevation_x(mesh: &PolyData) -> PolyData {
    let bounds = mesh.bounds();
    elevation(mesh, [bounds.x_min, 0.0, 0.0], [bounds.x_max, 0.0, 0.0])
}

pub fn elevation_y(mesh: &PolyData) -> PolyData {
    let bounds = mesh.bounds();
    elevation(mesh, [0.0, bounds.y_min, 0.0], [0.0, bounds.y_max, 0.0])
}

pub fn elevation_axis(mesh: &PolyData, axis: usize) -> PolyData {
    match axis {
        0 => elevation_x(mesh),
        1 => elevation_y(mesh),
        _ => elevation_z(mesh),
    }
}

pub fn elevation_along(mesh: &PolyData, direction: [f64; 3]) -> PolyData {
    elevation(mesh, [0.0, 0.0, 0.0], direction)
}

pub fn elevation(mesh: &PolyData, low_point: [f64; 3], high_point: [f64; 3]) -> PolyData {
    elevation_with_scalar_range(mesh, low_point, high_point, [0.0, 1.0])
}

pub fn elevation_with_scalar_range(
    mesh: &PolyData,
    low_point: [f64; 3],
    high_point: [f64; 3],
    scalar_range: [f64; 2],
) -> PolyData {
    let mut direction = [
        high_point[0] - low_point[0],
        high_point[1] - low_point[1],
        high_point[2] - low_point[2],
    ];
    let mut length2 =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if length2 <= 0.0 {
        direction = [0.0, 0.0, 1.0];
        length2 = 1.0;
    }
    let scalar_delta = scalar_range[1] - scalar_range[0];
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let v = [
                p[0] - low_point[0],
                p[1] - low_point[1],
                p[2] - low_point[2],
            ];
            let normalized = ((v[0] * direction[0] + v[1] * direction[1] + v[2] * direction[2])
                / length2)
                .clamp(0.0, 1.0);
            scalar_range[0] + normalized * scalar_delta
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
            .get_array("Elevation")
            .unwrap()
            .tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
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
