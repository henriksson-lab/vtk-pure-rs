use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply histogram equalization to an ImageData scalar field.
///
/// Remaps values so the cumulative histogram is approximately uniform,
/// improving contrast. Output range is [0, 1].
pub fn image_histogram_equalize(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }

    let mut buf = [0.0f64];
    let mut values: Vec<(f64, usize)> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0], i)
        })
        .collect();

    values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut result = vec![0.0f64; n];
    for (rank, &(_, orig_idx)) in values.iter().enumerate() {
        result[orig_idx] = rank as f64 / (n - 1).max(1) as f64;
    }

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equalizes_range() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        // Skewed distribution: [1, 1, 1, 1, 100]
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 1.0, 100.0],
                1,
            )));

        let result = image_histogram_equalize(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // The max value should map to 1.0
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        // The min values should be < 1.0
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.5);
    }

    #[test]
    fn uniform_unchanged() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let result = image_histogram_equalize(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_histogram_equalize(&img, "nope");
        assert_eq!(result.dimensions(), [3, 1, 1]);
    }
}
