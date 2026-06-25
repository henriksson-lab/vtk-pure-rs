use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a binary threshold to an ImageData scalar field.
///
/// All voxels with value above `threshold` are set to 1.0, and those at or
/// below are set to 0.0. Adds a "BinaryMask" point data array.
pub fn image_threshold_binary(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    let mut mask = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        mask[i] = if buf[0] > threshold { 1.0 } else { 0.0 };
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BinaryMask",
            mask,
            1,
        )));
    img
}

/// Apply an inverse binary threshold to an ImageData scalar field.
///
/// All voxels with value below `threshold` are set to 1.0, and those at or
/// above are set to 0.0. Adds a "BinaryMask" point data array.
pub fn image_threshold_binary_inverse(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    let mut mask = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        mask[i] = if buf[0] < threshold { 1.0 } else { 0.0 };
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BinaryMask",
            mask,
            1,
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
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn binary_threshold_above() {
        let img = make_image();
        let result = image_threshold_binary(&img, "val", 4.5);
        let arr = result.point_data().get_array("BinaryMask").unwrap();
        let mut buf = [0.0f64];

        // Values 0..4 should be 0.0, values 5..8 should be 1.0
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(8, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn binary_threshold_inverse() {
        let img = make_image();
        let result = image_threshold_binary_inverse(&img, "val", 4.5);
        let arr = result.point_data().get_array("BinaryMask").unwrap();
        let mut buf = [0.0f64];

        // Values 0..4 should be 1.0, values 5..8 should be 0.0
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_scalars_returns_clone() {
        let img = make_image();
        let result = image_threshold_binary(&img, "nonexistent", 5.0);
        assert!(result.point_data().get_array("BinaryMask").is_none());
        assert!(result.point_data().get_array("val").is_some());
    }
}
