use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the magnitude of a multi-component array on ImageData.
///
/// For each tuple in the named array, computes the Euclidean magnitude
/// (square root of the sum of squared components) and stores the result
/// as a 1-component "Magnitude" array in the output point data.
pub fn compute_magnitude(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc: usize = arr.num_components();
    let nt: usize = arr.num_tuples();
    let mut magnitudes: Vec<f64> = Vec::with_capacity(nt);
    let mut buf: Vec<f64> = vec![0.0; nc];

    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        let mut sum_sq: f64 = 0.0;
        for c in 0..nc {
            sum_sq += buf[c] * buf[c];
        }
        magnitudes.push(sum_sq.sqrt());
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Magnitude",
            magnitudes,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_four_five() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        // Two vectors: (3,4,0) and (0,0,5)
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Vectors",
                vec![3.0, 4.0, 0.0, 0.0, 0.0, 5.0],
                3,
            )));
        let result = compute_magnitude(&img, "Vectors");
        let mag = result.point_data().get_array("Magnitude").unwrap();
        assert_eq!(mag.num_components(), 1);
        assert_eq!(mag.num_tuples(), 2);
        let mut val = [0.0f64];
        mag.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10, "expected 5, got {}", val[0]);
        mag.tuple_as_f64(1, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10, "expected 5, got {}", val[0]);
    }

    #[test]
    fn single_component_is_abs() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Scalars",
                vec![-2.0, 0.0, 3.0],
                1,
            )));
        let result = compute_magnitude(&img, "Scalars");
        let mag = result.point_data().get_array("Magnitude").unwrap();
        let mut val = [0.0f64];
        mag.tuple_as_f64(0, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);
        mag.tuple_as_f64(1, &mut val);
        assert!((val[0]).abs() < 1e-10);
        mag.tuple_as_f64(2, &mut val);
        assert!((val[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = compute_magnitude(&img, "nonexistent");
        assert!(result.point_data().get_array("Magnitude").is_none());
    }
}
