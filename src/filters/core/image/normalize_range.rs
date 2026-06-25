use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};

/// Normalize scalar values in an ImageData to the [0, 1] range.
///
/// Reads the array named `scalars` from point data, finds the min and max,
/// then creates a new "Normalized" array with values linearly mapped to [0, 1].
/// If min == max, all output values are 0.0.
pub fn normalize_to_unit(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc: usize = arr.num_components();
    let nt: usize = arr.num_tuples();
    if nt == 0 {
        return input.clone();
    }

    // Read all values to find min and max.
    let total: usize = nt * nc;
    let mut values: Vec<f64> = vec![0.0; total];
    let mut buf: Vec<f64> = vec![0.0; nc];
    for t in 0..nt {
        arr.tuple_as_f64(t, &mut buf);
        for c in 0..nc {
            values[t * nc + c] = buf[c];
        }
    }

    let mut vmin: f64 = f64::MAX;
    let mut vmax: f64 = f64::NEG_INFINITY;
    for &v in &values {
        if v < vmin {
            vmin = v;
        }
        if v > vmax {
            vmax = v;
        }
    }

    let range: f64 = vmax - vmin;
    let normalized: Vec<f64> = if range.abs() < 1e-30 {
        vec![0.0; total]
    } else {
        values.iter().map(|&v| (v - vmin) / range).collect()
    };

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normalized",
            normalized,
            nc,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_simple() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        let data: Vec<f64> = vec![10.0, 20.0, 30.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", data, 1)));

        let result = normalize_to_unit(&img, "Scalars");
        let arr = result.point_data().get_array("Normalized").unwrap();
        assert_eq!(arr.num_tuples(), 3);

        let mut val = [0.0f64; 1];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 0.5).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn normalize_constant_field() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        let data: Vec<f64> = vec![5.0; 4];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", data, 1)));

        let result = normalize_to_unit(&img, "Scalars");
        let arr = result.point_data().get_array("Normalized").unwrap();
        let mut val = [0.0f64; 1];
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut val);
            assert!((val[0] - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn normalize_missing_array_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 1);
        let result = normalize_to_unit(&img, "DoesNotExist");
        assert_eq!(result.num_points(), 4);
    }
}
