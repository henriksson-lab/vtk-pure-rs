//! Logarithmic and exponential transforms for images.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Apply log transform.
///
/// Matches vtkImageLogarithmicScale: positive values use
/// `scale * ln(1 + value)`, and zero/negative values use
/// `-scale * ln(1 - value)`.
pub fn log_transform(input: &ImageData, scalars: &str, scale: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        data.extend(buf.iter().map(|&v| {
            if v > 0.0 {
                scale * (v + 1.0).ln()
            } else {
                -scale * (1.0 - v).ln()
            }
        }));
    }

    replace_point_array(
        input,
        scalars,
        AnyDataArray::F64(DataArray::from_vec(scalars, data, num_components)),
    )
}

/// Apply exponential transform: output = scale * (exp(value / scale) - 1).
pub fn exp_transform(input: &ImageData, scalars: &str, scale: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let s = if scale.abs() < 1e-15 { 1.0 } else { scale };
    let mut data = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        data.extend(buf.iter().map(|&v| s * ((v / s).exp() - 1.0)));
    }

    replace_point_array(
        input,
        scalars,
        AnyDataArray::F64(DataArray::from_vec(scalars, data, num_components)),
    )
}

/// Apply gamma correction: output = value^gamma (values normalized to [0,1]).
pub fn gamma_correct(input: &ImageData, scalars: &str, gamma: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut vals = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.extend_from_slice(&buf);
    }
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let norm = (v - mn) / range;
            norm.powf(gamma) * range + mn
        })
        .collect();

    replace_point_array(
        input,
        scalars,
        AnyDataArray::F64(DataArray::from_vec(scalars, data, num_components)),
    )
}

fn replace_point_array(input: &ImageData, scalars: &str, replacement: AnyDataArray) -> ImageData {
    let mut img = input.clone();
    let mut attrs = DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let array = input.point_data().get_array_by_index(i).unwrap();
        if array.name() == scalars {
            attrs.add_array(replacement.clone());
        } else {
            attrs.add_array(array.clone());
        }
    }
    *img.point_data_mut() = attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_log() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let result = log_transform(&img, "v", 1.0);
        assert_eq!(result.dimensions(), [5, 5, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0f64.ln()).abs() < 1e-10);
    }
    #[test]
    fn test_log_preserves_negative_sign() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![-3.0, 0.0, 3.0],
                1,
            )));

        let result = log_transform(&img, "v", 2.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] + 2.0 * 4.0f64.ln()).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 2.0 * 4.0f64.ln()).abs() < 1e-10);
    }
    #[test]
    fn test_log_transforms_all_components_and_preserves_arrays() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, -1.0, 3.0, -3.0],
                2,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "other",
                vec![42.0, 43.0],
                1,
            )));

        let result = log_transform(&img, "v", 1.0);
        assert!(result.point_data().get_array("other").is_some());
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0f64.ln()).abs() < 1e-10);
        assert!((buf[1] + 2.0f64.ln()).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 4.0f64.ln()).abs() < 1e-10);
        assert!((buf[1] + 4.0f64.ln()).abs() < 1e-10);
    }
    #[test]
    fn test_gamma() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x as f64,
        );
        let result = gamma_correct(&img, "v", 2.0);
        assert_eq!(result.dimensions(), [5, 5, 1]);
    }
    #[test]
    fn test_exp() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x as f64,
        );
        let result = exp_transform(&img, "v", 1.0);
        assert_eq!(result.dimensions(), [5, 5, 1]);
    }
}
