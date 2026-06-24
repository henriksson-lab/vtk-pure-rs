use crate::data::{AnyDataArray, DataArray, ImageData};

/// Clamp scalar values in an ImageData to the range [min, max].
///
/// Reads the named scalar array and produces a new "Clamped" array
/// where each value is clamped to the specified range.
pub fn clamp_scalars(input: &ImageData, scalars: &str, min: f64, max: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    let nc: usize = arr.num_components();
    let mut clamped: Vec<f64> = Vec::with_capacity(n * nc);
    let mut buf: Vec<f64> = vec![0.0; nc];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        for j in 0..nc {
            let v: f64 = buf[j];
            let c: f64 = if v < min {
                min
            } else if v > max {
                max
            } else {
                v
            };
            clamped.push(c);
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Clamped", clamped, nc,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image() -> ImageData {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let values: Vec<f64> = (0..9).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalars", values, 1)));
        img
    }

    #[test]
    fn clamp_range() {
        let img = make_image();
        let result = clamp_scalars(&img, "scalars", 2.0, 6.0);
        let arr = result.point_data().get_array("Clamped").unwrap();
        assert_eq!(arr.num_tuples(), 9);
        let mut buf = [0.0f64];
        // 0 -> clamped to 2
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
        // 4 -> stays 4
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 4.0).abs() < 1e-10);
        // 8 -> clamped to 6
        arr.tuple_as_f64(8, &mut buf);
        assert!((buf[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn clamp_preserves_original() {
        let img = make_image();
        let result = clamp_scalars(&img, "scalars", 2.0, 6.0);
        // Original array still present
        let orig = result.point_data().get_array("scalars").unwrap();
        let mut buf = [0.0f64];
        orig.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn clamp_missing_array_returns_clone() {
        let img = make_image();
        let result = clamp_scalars(&img, "nonexistent", 0.0, 1.0);
        assert!(result.point_data().get_array("Clamped").is_none());
        assert!(result.point_data().get_array("scalars").is_some());
    }
}
