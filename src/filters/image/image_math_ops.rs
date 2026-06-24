//! Pixel-wise math operations between images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Add two images pixel-wise.
pub fn image_add(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| x + y)
}

/// Subtract b from a pixel-wise.
pub fn image_subtract(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| x - y)
}

/// Multiply two images pixel-wise.
pub fn image_multiply(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| x * y)
}

/// Divide a by b pixel-wise (safe: returns 0 for zero divisor).
pub fn image_divide(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| if y.abs() > 1e-30 { x / y } else { 0.0 })
}

/// Max of two images pixel-wise.
pub fn image_max(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| x.max(y))
}

/// Min of two images pixel-wise.
pub fn image_min(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| x.min(y))
}

/// Weighted blend: result = alpha * a + (1-alpha) * b.
pub fn image_blend_weighted(a: &ImageData, b: &ImageData, name: &str, alpha: f64) -> ImageData {
    binary_op(a, b, name, move |x, y| alpha * x + (1.0 - alpha) * y)
}

fn binary_op(a: &ImageData, b: &ImageData, name: &str, f: impl Fn(f64, f64) -> f64) -> ImageData {
    let arr_a = match a.point_data().get_array(name) {
        Some(x) if x.num_components() == 1 => x,
        _ => return a.clone(),
    };
    let arr_b = match b.point_data().get_array(name) {
        Some(x) if x.num_components() == 1 => x,
        _ => return a.clone(),
    };
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ba = [0.0f64];
    let mut bb = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut ba);
            arr_b.tuple_as_f64(i, &mut bb);
            f(ba[0], bb[0])
        })
        .collect();
    let dims = a.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(a.spacing())
        .with_origin(a.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() {
        let a = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let b = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, y, _| y,
        );
        let r = image_add(&a, &b, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(3 + 2 * 5, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10); // 3 + 2
    }
    #[test]
    fn test_blend() {
        let a = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 100.0,
        );
        let b = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let r = image_blend_weighted(&a, &b, "v", 0.3);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 30.0).abs() < 1e-10);
    }
}
