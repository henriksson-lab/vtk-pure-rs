//! Threshold-based texture coordinate generation.
//!
//! Generates texture coordinates based on scalar value thresholds,
//! mapping scalar ranges to UV regions.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate texture coordinates from scalar values.
///
/// Maps scalar values linearly to U coordinate in [0, 1].
/// V is set to 0.5 (for 1D texture lookup / color ramp).
pub fn texture_coords_from_scalar(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_val = min_val.min(buf[0]);
        max_val = max_val.max(buf[0]);
    }

    let range = max_val - min_val;
    let mut tcoords = Vec::with_capacity(n * 2);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let u = if range > 1e-15 {
            (buf[0] - min_val) / range
        } else {
            0.5
        };
        tcoords.push(u);
        tcoords.push(0.5);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

/// Generate texture coordinates with a custom range mapping.
pub fn texture_coords_from_scalar_range(
    mesh: &PolyData,
    array_name: &str,
    scalar_min: f64,
    scalar_max: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };

    let n = arr.num_tuples();
    let range = scalar_max - scalar_min;
    let mut tcoords = Vec::with_capacity(n * 2);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let u = if range.abs() > 1e-15 {
            ((buf[0] - scalar_min) / range).clamp(0.0, 1.0)
        } else {
            0.5
        };
        tcoords.push(u);
        tcoords.push(0.5);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

/// Generate binary texture coordinates: below threshold → u=0, above → u=1.
pub fn texture_coords_binary_threshold(
    mesh: &PolyData,
    array_name: &str,
    threshold: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };

    let n = arr.num_tuples();
    let mut tcoords = Vec::with_capacity(n * 2);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let u = if buf[0] >= threshold { 1.0 } else { 0.0 };
        tcoords.push(u);
        tcoords.push(0.5);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh() -> PolyData {
        let mut mesh =
            PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 50.0, 100.0],
                1,
            )));
        mesh
    }

    #[test]
    fn from_scalar() {
        let result = texture_coords_from_scalar(&make_mesh(), "val");
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        tc.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn custom_range() {
        let result = texture_coords_from_scalar_range(&make_mesh(), "val", 0.0, 200.0);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10); // 100/200 = 0.5
    }

    #[test]
    fn binary() {
        let result = texture_coords_binary_threshold(&make_mesh(), "val", 50.0);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        tc.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0); // 50 >= 50
    }

    #[test]
    fn missing_array() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let result = texture_coords_from_scalar(&mesh, "nonexistent");
        assert!(result.point_data().tcoords().is_none());
    }
}
