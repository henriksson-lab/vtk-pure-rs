//! Compute distance field from mesh centroid, useful for radial analysis.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn radial_distance_field(mesh: &PolyData) -> PolyData {
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
            "RadialDist",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("RadialDist");
    r
}
pub fn angular_field(mesh: &PolyData, axis: usize) -> PolyData {
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
            match axis {
                0 => (p[2] - cz).atan2(p[1] - cy),
                1 => (p[0] - cx).atan2(p[2] - cz),
                _ => (p[1] - cy).atan2(p[0] - cx),
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Angle", data, 1)));
    r.point_data_mut().set_active_scalars("Angle");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_radial() {
        let m = PolyData::from_triangles(
            vec![[3.0, 4.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = radial_distance_field(&m);
        assert!(r.point_data().get_array("RadialDist").is_some());
    }
    #[test]
    fn test_angular() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = angular_field(&m, 2);
        assert!(r.point_data().get_array("Angle").is_some());
    }
}
