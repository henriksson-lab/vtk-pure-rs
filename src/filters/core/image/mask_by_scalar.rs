use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Mask an ImageData by scalar range.
///
/// Keeps voxels where the named scalar array value is within `[min, max]`
/// (inclusive). Voxels outside this range are set to `fill`.
pub fn mask_by_scalar_range(
    input: &ImageData,
    scalars: &str,
    min: f64,
    max: f64,
    fill: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    let nc: usize = arr.num_components();
    let mut result: Vec<f64> = Vec::with_capacity(n * nc);
    let mut buf: Vec<f64> = vec![0.0; nc];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        result.extend(
            buf.iter()
                .map(|&val| if val >= min && val <= max { val } else { fill }),
        );
    }

    let mut img = input.clone();
    let mut new_attrs = DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                nc,
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

    fn make_test_image() -> ImageData {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        // Values 1..9
        let values: Vec<f64> = (1..=9).map(|x| x as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalars", values, 1)));
        img
    }

    #[test]
    fn mask_keeps_in_range() {
        let img = make_test_image();
        let result = mask_by_scalar_range(&img, "scalars", 3.0, 7.0, 0.0);
        let arr = result.point_data().get_array("scalars").unwrap();
        let mut val = [0.0f64];

        // Value 1 (out of range) -> fill
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 0.0).abs() < 1e-10);

        // Value 5 (in range) -> kept
        arr.tuple_as_f64(4, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10);

        // Value 9 (out of range) -> fill
        arr.tuple_as_f64(8, &mut val);
        assert!((val[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn mask_boundary_inclusive() {
        let img = make_test_image();
        let result = mask_by_scalar_range(&img, "scalars", 3.0, 7.0, -1.0);
        let arr = result.point_data().get_array("scalars").unwrap();
        let mut val = [0.0f64];

        // Value 3 (boundary) -> kept
        arr.tuple_as_f64(2, &mut val);
        assert!((val[0] - 3.0).abs() < 1e-10);

        // Value 7 (boundary) -> kept
        arr.tuple_as_f64(6, &mut val);
        assert!((val[0] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = make_test_image();
        let result = mask_by_scalar_range(&img, "nope", 0.0, 10.0, 0.0);
        assert!(result.point_data().get_array("scalars").is_some());
    }

    #[test]
    fn multi_component_thresholds_each_component() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "scalars",
                vec![1.0, 5.0, 8.0, 3.0],
                2,
            )));
        let result = mask_by_scalar_range(&img, "scalars", 3.0, 7.0, -1.0);
        let arr = result.point_data().get_array("scalars").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut val = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut val);
        assert_eq!(val, [-1.0, 5.0]);
        arr.tuple_as_f64(1, &mut val);
        assert_eq!(val, [-1.0, 3.0]);
    }
}
