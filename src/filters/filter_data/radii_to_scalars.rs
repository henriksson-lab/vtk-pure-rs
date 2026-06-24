//! Convert point radii (distance from origin or center) to scalar field.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute distance from each point to a center and add as scalar array.
pub fn radii_to_scalars(mesh: &PolyData, center: [f64; 3], array_name: &str) -> PolyData {
    let n = mesh.points.len();
    let mut radii = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        let r =
            ((p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2))
                .sqrt();
        radii.push(r);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, radii, 1)));
    result
}

/// Compute distance from each point to the mesh centroid.
pub fn radii_from_centroid(mesh: &PolyData, array_name: &str) -> PolyData {
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
    radii_to_scalars(mesh, [cx / nf, cy / nf, cz / nf], array_name)
}

/// Compute normalized radii (0 at center, 1 at max distance).
pub fn radii_normalized(mesh: &PolyData, center: [f64; 3], array_name: &str) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut radii = Vec::with_capacity(n);
    let mut max_r = 0.0f64;
    for i in 0..n {
        let p = mesh.points.get(i);
        let r =
            ((p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2))
                .sqrt();
        radii.push(r);
        max_r = max_r.max(r);
    }
    if max_r > 1e-15 {
        for r in &mut radii {
            *r /= max_r;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, radii, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_radii() {
        let mesh = PolyData::from_points(vec![[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]);
        let result = radii_to_scalars(&mesh, [0.0, 0.0, 0.0], "Radius");
        let arr = result.point_data().get_array("Radius").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn from_centroid() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = radii_from_centroid(&mesh, "R");
        let arr = result.point_data().get_array("R").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10); // distance from centroid (1,0,0) to (0,0,0)
    }

    #[test]
    fn normalized() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = radii_normalized(&mesh, [0.0, 0.0, 0.0], "NormR");
        let arr = result.point_data().get_array("NormR").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }
}
