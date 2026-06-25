use crate::data::{AnyDataArray, DataArray, ImageData};

/// Threshold an ImageData scalar field, replacing values outside [lower, upper]
/// with a replacement value.
///
/// Modifies the named scalar array in-place. Values inside the range are
/// kept; values outside are set to `replacement`.
pub fn image_threshold(
    input: &ImageData,
    scalars: &str,
    lower: f64,
    upper: f64,
    replacement: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = if buf[0] >= lower && buf[0] <= upper {
            buf[0]
        } else {
            replacement
        };
    }

    let mut img = input.clone();
    // Replace the array
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

/// Create a binary mask from an ImageData scalar field.
///
/// Adds a "Mask" array where values in [lower, upper] are 1.0 and others 0.0.
pub fn image_binary_threshold(
    input: &ImageData,
    scalars: &str,
    lower: f64,
    upper: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut mask = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        mask[i] = if buf[0] >= lower && buf[0] <= upper {
            1.0
        } else {
            0.0
        };
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Mask", mask, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image() -> ImageData {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let values: Vec<f64> = (0..9).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn threshold_range() {
        let img = make_image();
        let result = image_threshold(&img, "val", 3.0, 6.0, 0.0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // below range -> replacement
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 4.0); // in range -> kept
        arr.tuple_as_f64(8, &mut buf);
        assert_eq!(buf[0], 0.0); // above range -> replacement
    }

    #[test]
    fn binary_mask() {
        let img = make_image();
        let result = image_binary_threshold(&img, "val", 3.0, 5.0);
        let arr = result.point_data().get_array("Mask").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_scalars() {
        let img = make_image();
        let result = image_threshold(&img, "nope", 0.0, 1.0, 0.0);
        assert!(result.point_data().get_array("val").is_some());
    }
}
