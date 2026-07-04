//! Pixel-level operations on ImageData: threshold, clamp, remap, quantize.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply per-pixel function.
pub fn pixel_map(image: &ImageData, array_name: &str, f: impl Fn(f64) -> f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let mut data = Vec::with_capacity(arr.num_tuples() * nc);
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        data.extend(buf.iter().map(|v| f(*v)));
    }
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, nc)));
    r
}

/// Apply per-pixel binary function with two arrays.
pub fn pixel_binary_op(
    image: &ImageData,
    a_name: &str,
    b_name: &str,
    result_name: &str,
    f: impl Fn(f64, f64) -> f64,
) -> ImageData {
    let a = match image.point_data().get_array(a_name) {
        Some(x) => x,
        None => return image.clone(),
    };
    let b = match image.point_data().get_array(b_name) {
        Some(x) => x,
        None => return image.clone(),
    };
    if a.num_components() != b.num_components() {
        return image.clone();
    }
    let n = a.num_tuples().min(b.num_tuples());
    let nc = a.num_components();
    let mut ab = vec![0.0f64; nc];
    let mut bb = vec![0.0f64; nc];
    let mut data = Vec::with_capacity(n * nc);
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        data.extend(ab.iter().zip(bb.iter()).map(|(a, b)| f(*a, *b)));
    }
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            data,
            nc,
        )));
    r
}

/// Threshold: values below→0, above→1.
pub fn binary_threshold(image: &ImageData, array_name: &str, threshold: f64) -> ImageData {
    pixel_map(
        image,
        array_name,
        |v| if v >= threshold { 1.0 } else { 0.0 },
    )
}

/// Invert values: result = max - value + min.
pub fn invert_values(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let mut buf = vec![0.0f64; arr.num_components()];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        for value in &buf {
            min_v = min_v.min(*value);
            max_v = max_v.max(*value);
        }
    }
    pixel_map(image, array_name, move |v| max_v - v + min_v)
}

/// Absolute value.
pub fn abs_values(image: &ImageData, array_name: &str) -> ImageData {
    pixel_map(image, array_name, |v| v.abs())
}

/// Square root.
pub fn sqrt_values(image: &ImageData, array_name: &str) -> ImageData {
    pixel_map(image, array_name, |v| v.sqrt())
}

/// Power: v^exponent.
pub fn pow_values(image: &ImageData, array_name: &str, exponent: f64) -> ImageData {
    pixel_map(image, array_name, move |v| v.powf(exponent))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn threshold() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = binary_threshold(&img, "v", 5.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(7, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn invert() {
        let img = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = invert_values(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 4.0).abs() < 0.01);
    }
    #[test]
    fn binary_op() {
        let mut img = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "a",
            |x, _, _| x,
        );
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![1.0; 5], 1)));
        let result = pixel_binary_op(&img, "a", "b", "c", |a, b| a + b);
        assert!(result.point_data().get_array("c").is_some());
    }
    #[test]
    fn sqrt() {
        let img = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 4.0,
        );
        let result = sqrt_values(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 0.01);
    }

    #[test]
    fn map_preserves_components() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                2,
            )));
        let result = abs_values(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
    }
}
