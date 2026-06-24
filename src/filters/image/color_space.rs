//! Color space conversions for multi-component image data.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Convert RGB image (3-component array) to grayscale (1-component).
pub fn rgb_to_grayscale(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            0.299 * buf[0] + 0.587 * buf[1] + 0.114 * buf[2]
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Grayscale", data, 1)))
}

/// Convert RGB to HSV (each component in [0,1] assuming input in [0,255]).
pub fn rgb_to_hsv(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let r = buf[0] / 255.0;
        let g = buf[1] / 255.0;
        let b = buf[2] / 255.0;
        let mx = r.max(g).max(b);
        let mn = r.min(g).min(b);
        let d = mx - mn;
        let h = if d < 1e-15 {
            0.0
        } else if (mx - r).abs() < 1e-15 {
            (((g - b) / d) % 6.0) / 6.0
        } else if (mx - g).abs() < 1e-15 {
            ((b - r) / d + 2.0) / 6.0
        } else {
            ((r - g) / d + 4.0) / 6.0
        };
        let s = if mx < 1e-15 { 0.0 } else { d / mx };
        let v = mx;
        data.push(h.rem_euclid(1.0));
        data.push(s);
        data.push(v);
    }
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("HSV", data, 3)))
}

/// Convert HSV ([0,1] range) back to RGB ([0,255] range).
pub fn hsv_to_rgb(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let h = buf[0] * 6.0;
        let s = buf[1];
        let v = buf[2];
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        data.push((r + m) * 255.0);
        data.push((g + m) * 255.0);
        data.push((b + m) * 255.0);
    }
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("RGB", data, 3)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rgb_gray() {
        let dims = [4, 4, 1];
        let data: Vec<f64> = (0..16)
            .flat_map(|i| vec![i as f64 * 16.0, i as f64 * 8.0, i as f64 * 4.0])
            .collect();
        let img = ImageData::with_dimensions(dims[0], dims[1], dims[2])
            .with_spacing([1.0, 1.0, 1.0])
            .with_origin([0.0, 0.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("RGB", data, 3)));
        let gray = rgb_to_grayscale(&img, "RGB");
        assert!(gray.point_data().get_array("Grayscale").is_some());
        let arr = gray.point_data().get_array("Grayscale").unwrap();
        assert_eq!(arr.num_components(), 1);
    }
    #[test]
    fn test_rgb_hsv_roundtrip() {
        let data: Vec<f64> = vec![
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 128.0, 128.0, 128.0,
        ];
        let img = ImageData::with_dimensions(4, 1, 1)
            .with_spacing([1.0, 1.0, 1.0])
            .with_origin([0.0, 0.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "RGB",
                data.clone(),
                3,
            )));
        let hsv = rgb_to_hsv(&img, "RGB");
        let back = hsv_to_rgb(&hsv, "HSV");
        let arr = back.point_data().get_array("RGB").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 255.0).abs() < 1.0); // red
    }
}
