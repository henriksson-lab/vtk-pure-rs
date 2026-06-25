//! Image arithmetic: element-wise add, subtract, multiply, divide, max, min.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Element-wise add two image arrays.
pub fn image_add(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_arith(a, b, name, |x, y| x + y)
}

/// Element-wise subtract.
pub fn image_subtract(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_arith(a, b, name, |x, y| x - y)
}

/// Element-wise multiply.
pub fn image_multiply(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_arith(a, b, name, |x, y| x * y)
}

/// Element-wise divide (with zero protection).
pub fn image_divide(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_arith(a, b, name, |x, y| if y != 0.0 { x / y } else { f64::MAX })
}

/// Element-wise maximum.
pub fn image_max(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_arith(a, b, name, |x, y| x.max(y))
}

/// Element-wise minimum.
pub fn image_min(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_arith(a, b, name, |x, y| x.min(y))
}

/// Scale an array by a constant.
pub fn image_scale(image: &ImageData, name: &str, factor: f64) -> ImageData {
    unary_arith(image, name, |x| x * factor)
}

/// Add a constant to an array.
pub fn image_offset(image: &ImageData, name: &str, offset: f64) -> ImageData {
    unary_arith(image, name, |x| x + offset)
}

/// Absolute value.
pub fn image_abs(image: &ImageData, name: &str) -> ImageData {
    unary_arith(image, name, |x| x.abs())
}

/// Square root.
pub fn image_sqrt(image: &ImageData, name: &str) -> ImageData {
    unary_arith(image, name, |x| x.sqrt())
}

/// Natural logarithm.
pub fn image_ln(image: &ImageData, name: &str) -> ImageData {
    unary_arith(image, name, |x| x.ln())
}

/// Exponential.
pub fn image_exp(image: &ImageData, name: &str) -> ImageData {
    unary_arith(image, name, |x| x.exp())
}

fn binary_arith(
    a: &ImageData,
    b: &ImageData,
    name: &str,
    f: impl Fn(f64, f64) -> f64,
) -> ImageData {
    let aa = match a.point_data().get_array(name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let ba = match b.point_data().get_array(name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let num_components = aa.num_components();
    if ba.num_components() != num_components {
        return a.clone();
    }
    let n = aa.num_tuples().min(ba.num_tuples());
    let mut ab = vec![0.0f64; num_components];
    let mut bb = vec![0.0f64; num_components];
    let d: Vec<f64> = (0..n)
        .flat_map(|i| {
            aa.tuple_as_f64(i, &mut ab);
            ba.tuple_as_f64(i, &mut bb);
            ab.iter()
                .zip(&bb)
                .map(|(&x, &y)| f(x, y))
                .collect::<Vec<_>>()
        })
        .collect();
    let mut r = a.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            name,
            d,
            num_components,
        )));
    r
}

fn unary_arith(img: &ImageData, name: &str, f: impl Fn(f64) -> f64) -> ImageData {
    let arr = match img.point_data().get_array(name) {
        Some(x) => x,
        None => return img.clone(),
    };
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let d: Vec<f64> = (0..arr.num_tuples())
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.iter().map(|&v| f(v)).collect::<Vec<_>>()
        })
        .collect();
    let mut r = img.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            name,
            d,
            num_components,
        )));
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add() {
        let a = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 3.0,
        );
        let b = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 2.0,
        );
        let r = image_add(&a, &b, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 0.01);
    }
    #[test]
    fn scale() {
        let img = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 4.0,
        );
        let r = image_scale(&img, "v", 0.5);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 0.01);
    }
    #[test]
    fn divide_zero() {
        let a = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 5.0,
        );
        let b = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let r = image_divide(&a, &b, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], f64::MAX);
    }
}
