use crate::data::{AnyDataArray, DataArray, ImageData};

/// Invert scalar values on an ImageData.
///
/// For each value, computes `new_val = max - val + min`, effectively flipping
/// the value range. The result is stored as a new array named "Inverted".
///
/// Returns a clone of the input if the named array is not found.
pub fn invert_scalars(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }

    // First pass: read values and find min/max
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    let mut min_val: f64 = f64::MAX;
    let mut max_val: f64 = f64::MIN;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
        if buf[0] < min_val {
            min_val = buf[0];
        }
        if buf[0] > max_val {
            max_val = buf[0];
        }
    }

    // Second pass: invert
    let inverted: Vec<f64> = values.iter().map(|&v| max_val - v + min_val).collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Inverted", inverted, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    fn make_image_with_scalars(values: Vec<f64>) -> ImageData {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", values, 1)));
        img
    }

    #[test]
    fn test_invert_basic() {
        let img = make_image_with_scalars(vec![1.0, 2.0, 3.0, 4.0]);
        let result = invert_scalars(&img, "Scalars");
        let arr = result.point_data().get_array("Inverted").unwrap();
        assert_eq!(arr.num_tuples(), 4);

        let mut buf = [0.0f64];
        // max=4, min=1: inverted = 4 - val + 1 = 5 - val
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 4.0).abs() < 1e-10); // 5 - 1 = 4

        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10); // 5 - 2 = 3

        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10); // 5 - 3 = 2

        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10); // 5 - 4 = 1
    }

    #[test]
    fn test_invert_missing_array() {
        let img = make_image_with_scalars(vec![1.0, 2.0, 3.0, 4.0]);
        let result = invert_scalars(&img, "NonExistent");
        // Should return clone without "Inverted" array
        assert!(result.point_data().get_array("Inverted").is_none());
    }

    #[test]
    fn test_invert_constant_values() {
        let img = make_image_with_scalars(vec![5.0, 5.0, 5.0, 5.0]);
        let result = invert_scalars(&img, "Scalars");
        let arr = result.point_data().get_array("Inverted").unwrap();

        let mut buf = [0.0f64];
        // max=5, min=5: inverted = 5 - 5 + 5 = 5
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 5.0).abs() < 1e-10);
        }
    }
}
