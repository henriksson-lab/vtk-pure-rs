//! Attach distance from centroid as point scalar.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn distance_to_centroid(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DistToCentroid",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("DistToCentroid");
    r
}
pub fn distance_to_origin(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DistToOrigin",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("DistToOrigin");
    r
}
pub fn distance_to_axis(mesh: &PolyData, axis: usize) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            match axis {
                0 => (p[1] * p[1] + p[2] * p[2]).sqrt(),
                1 => (p[0] * p[0] + p[2] * p[2]).sqrt(),
                _ => (p[0] * p[0] + p[1] * p[1]).sqrt(),
            }
        })
        .collect();
    let name = match axis {
        0 => "DistToXAxis",
        1 => "DistToYAxis",
        _ => "DistToZAxis",
    };
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)));
    r.point_data_mut().set_active_scalars(name);
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_centroid() {
        let m = PolyData::from_triangles(
            vec![[3.0, 4.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_centroid(&m);
        assert!(r.point_data().get_array("DistToCentroid").is_some());
    }
    #[test]
    fn test_origin() {
        let m = PolyData::from_triangles(
            vec![[3.0, 4.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_origin(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("DistToOrigin")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_axis() {
        let m = PolyData::from_triangles(
            vec![[0.0, 3.0, 4.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_axis(&m, 0);
        let mut buf = [0.0];
        r.point_data()
            .get_array("DistToXAxis")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
}
