//! Threshold-based texture coordinate generation.

use crate::data::{AnyDataArray, DataArray, PolyData};

const DEFAULT_IN_TEXTURE_COORD: [f64; 3] = [0.75, 0.0, 0.0];
const DEFAULT_OUT_TEXTURE_COORD: [f64; 3] = [0.25, 0.0, 0.0];
const DEFAULT_TEXTURE_DIMENSION: usize = 2;

/// Criterion used by VTK's threshold texture coordinate filter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThresholdFunction {
    Lower(f64),
    Upper(f64),
    Between(f64, f64),
}

impl ThresholdFunction {
    fn contains(self, scalar: f64) -> bool {
        match self {
            ThresholdFunction::Lower(lower) => scalar <= lower,
            ThresholdFunction::Upper(upper) => scalar >= upper,
            ThresholdFunction::Between(lower, upper) => scalar >= lower && scalar <= upper,
        }
    }
}

/// Generate VTK-style threshold texture coordinates from point scalars.
///
/// Points satisfying `threshold_function` receive `in_texture_coord`; all
/// others receive `out_texture_coord`. `texture_dimension` is clamped to 1..=3.
pub fn threshold_texture_coords(
    mesh: &PolyData,
    array_name: &str,
    threshold_function: ThresholdFunction,
    texture_dimension: usize,
    in_texture_coord: [f64; 3],
    out_texture_coord: [f64; 3],
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() >= 1 => a,
        _ => return mesh.clone(),
    };

    let texture_dimension = texture_dimension.clamp(1, 3);
    let n = arr.num_tuples();
    let mut tcoords = Vec::with_capacity(n * texture_dimension);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let coord = if threshold_function.contains(buf[0]) {
            in_texture_coord
        } else {
            out_texture_coord
        };
        tcoords.extend_from_slice(&coord[..texture_dimension]);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords",
            tcoords,
            texture_dimension,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

/// VTK default: threshold by upper value 1.0 and emit 2D in/out coordinates.
pub fn texture_coords_from_scalar(mesh: &PolyData, array_name: &str) -> PolyData {
    threshold_texture_coords(
        mesh,
        array_name,
        ThresholdFunction::Upper(1.0),
        DEFAULT_TEXTURE_DIMENSION,
        DEFAULT_IN_TEXTURE_COORD,
        DEFAULT_OUT_TEXTURE_COORD,
    )
}

/// Generate threshold texture coordinates for values between lower and upper.
pub fn texture_coords_from_scalar_range(
    mesh: &PolyData,
    array_name: &str,
    scalar_min: f64,
    scalar_max: f64,
) -> PolyData {
    threshold_texture_coords(
        mesh,
        array_name,
        ThresholdFunction::Between(scalar_min, scalar_max),
        DEFAULT_TEXTURE_DIMENSION,
        DEFAULT_IN_TEXTURE_COORD,
        DEFAULT_OUT_TEXTURE_COORD,
    )
}

/// Generate threshold texture coordinates for values greater than or equal to threshold.
pub fn texture_coords_binary_threshold(
    mesh: &PolyData,
    array_name: &str,
    threshold: f64,
) -> PolyData {
    threshold_texture_coords(
        mesh,
        array_name,
        ThresholdFunction::Upper(threshold),
        DEFAULT_TEXTURE_DIMENSION,
        DEFAULT_IN_TEXTURE_COORD,
        DEFAULT_OUT_TEXTURE_COORD,
    )
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
        assert!((buf[0] - 0.25).abs() < 1e-10);
        tc.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.75).abs() < 1e-10);
    }

    #[test]
    fn custom_range() {
        let result = texture_coords_from_scalar_range(&make_mesh(), "val", 0.0, 200.0);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.75).abs() < 1e-10);
    }

    #[test]
    fn binary() {
        let result = texture_coords_binary_threshold(&make_mesh(), "val", 50.0);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.25);
        tc.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.75);
    }

    #[test]
    fn custom_dimension_and_coordinates() {
        let result = threshold_texture_coords(
            &make_mesh(),
            "val",
            ThresholdFunction::Lower(0.0),
            3,
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
        );
        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_components(), 3);
        let mut buf = [0.0f64; 3];
        tc.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 2.0, 3.0]);
        tc.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [4.0, 5.0, 6.0]);
    }

    #[test]
    fn missing_array() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let result = texture_coords_from_scalar(&mesh, "nonexistent");
        assert!(result.point_data().tcoords().is_none());
    }
}
