//! Band thresholding and multi-level quantization.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Band threshold: keep values in [lo, hi], zero everything else.
pub fn band_threshold(input: &ImageData, scalars: &str, lo: f64, hi: f64) -> ImageData {
    apply_map(input, scalars, |v| if v >= lo && v <= hi { v } else { 0.0 })
}

/// Quantize image to N discrete levels.
pub fn quantize(input: &ImageData, scalars: &str, levels: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };
    let lf = levels.max(1) as f64;
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let norm = ((v - mn) / range * lf).floor().min(lf - 1.0);
            mn + norm / (lf - 1.0).max(1.0) * range
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Apply sigmoid contrast curve: output = 1 / (1 + exp(-gain * (value - midpoint))).
pub fn sigmoid_contrast(input: &ImageData, scalars: &str, midpoint: f64, gain: f64) -> ImageData {
    apply_map(input, scalars, |v| {
        1.0 / (1.0 + (-gain * (v - midpoint)).exp())
    })
}

fn apply_map(input: &ImageData, scalars: &str, f: impl Fn(f64) -> f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            f(buf[0])
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_band() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = band_threshold(&img, "v", 3.0, 6.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(5, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_quantize() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = quantize(&img, "v", 3);
        assert_eq!(r.dimensions(), [10, 1, 1]);
    }
    #[test]
    fn test_sigmoid() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = sigmoid_contrast(&img, "v", 5.0, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10); // sigmoid(0) = 0.5
    }
}
