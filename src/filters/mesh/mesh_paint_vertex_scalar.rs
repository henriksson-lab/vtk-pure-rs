//! Paint scalar values onto vertices near a point.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn paint_scalar_sphere(
    mesh: &PolyData,
    array_name: &str,
    center: [f64; 3],
    radius: f64,
    value: f64,
) -> PolyData {
    let n = mesh.points.len();
    let r2 = radius * radius;
    let existing = mesh.point_data().get_array(array_name);
    let mut buf = [0.0f64];
    let mut data: Vec<f64> = if let Some(arr) = existing {
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            })
            .collect()
    } else {
        vec![0.0; n]
    };
    for i in 0..n {
        let p = mesh.points.get(i);
        let d2 =
            (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2);
        if d2 <= r2 {
            let t = 1.0 - (d2 / r2).sqrt();
            data[i] = data[i] * (1.0 - t) + value * t;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}
pub fn paint_scalar_line(
    mesh: &PolyData,
    array_name: &str,
    p0: [f64; 3],
    p1: [f64; 3],
    radius: f64,
    value: f64,
) -> PolyData {
    let n = mesh.points.len();
    let r2 = radius * radius;
    let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let dl2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let existing = mesh.point_data().get_array(array_name);
    let mut buf = [0.0f64];
    let mut data: Vec<f64> = if let Some(arr) = existing {
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            })
            .collect()
    } else {
        vec![0.0; n]
    };
    for i in 0..n {
        let p = mesh.points.get(i);
        let v = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let t = if dl2 > 1e-30 {
            (v[0] * d[0] + v[1] * d[1] + v[2] * d[2]) / dl2
        } else {
            0.0
        };
        let t = t.clamp(0.0, 1.0);
        let proj = [p0[0] + t * d[0], p0[1] + t * d[1], p0[2] + t * d[2]];
        let d2 = (p[0] - proj[0]).powi(2) + (p[1] - proj[1]).powi(2) + (p[2] - proj[2]).powi(2);
        if d2 <= r2 {
            let w = 1.0 - (d2 / r2).sqrt();
            data[i] = data[i] * (1.0 - w) + value * w;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sphere() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = paint_scalar_sphere(&m, "paint", [0.0, 0.0, 0.0], 0.5, 10.0);
        let arr = r.point_data().get_array("paint").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 5.0);
    }
    #[test]
    fn test_line() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = paint_scalar_line(&m, "paint", [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.1, 5.0);
        assert!(r.point_data().get_array("paint").is_some());
    }
}
