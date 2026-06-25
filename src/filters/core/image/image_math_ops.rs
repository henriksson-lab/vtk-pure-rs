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

/// Divide a by b pixel-wise.
///
/// Matches `vtkImageMathematics` default divide-by-zero behavior: when the
/// divisor is zero, the output value is the scalar type maximum.
pub fn image_divide(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    image_divide_with_constant(a, b, name, false, 0.0)
}

/// Divide a by b pixel-wise, optionally replacing divide-by-zero with c.
pub fn image_divide_with_constant(
    a: &ImageData,
    b: &ImageData,
    name: &str,
    divide_by_zero_to_c: bool,
    constant_c: f64,
) -> ImageData {
    binary_op(a, b, name, move |x, y| {
        if y != 0.0 {
            x / y
        } else if divide_by_zero_to_c {
            constant_c
        } else {
            f64::MAX
        }
    })
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

/// Set each output pixel to 1 / input.
pub fn image_invert(input: &ImageData, name: &str) -> ImageData {
    image_invert_with_constant(input, name, false, 0.0)
}

/// Invert, optionally replacing divide-by-zero with c.
pub fn image_invert_with_constant(
    input: &ImageData,
    name: &str,
    divide_by_zero_to_c: bool,
    constant_c: f64,
) -> ImageData {
    unary_op(input, name, move |x| {
        if x != 0.0 {
            1.0 / x
        } else if divide_by_zero_to_c {
            constant_c
        } else {
            f64::MAX
        }
    })
}

pub fn image_sin(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::sin)
}

pub fn image_cos(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::cos)
}

pub fn image_exp(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::exp)
}

pub fn image_log(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::ln)
}

pub fn image_abs(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::abs)
}

pub fn image_square(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, |x| x * x)
}

pub fn image_sqrt(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::sqrt)
}

pub fn image_atan(input: &ImageData, name: &str) -> ImageData {
    unary_op(input, name, f64::atan)
}

pub fn image_atan2(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    binary_op(a, b, name, |x, y| {
        if x == 0.0 && y == 0.0 {
            0.0
        } else {
            x.atan2(y)
        }
    })
}

pub fn image_multiply_by_k(input: &ImageData, name: &str, constant_k: f64) -> ImageData {
    unary_op(input, name, move |x| constant_k * x)
}

pub fn image_add_constant(input: &ImageData, name: &str, constant_c: f64) -> ImageData {
    unary_op(input, name, move |x| constant_c + x)
}

pub fn image_replace_c_by_k(
    input: &ImageData,
    name: &str,
    constant_c: f64,
    constant_k: f64,
) -> ImageData {
    unary_op(
        input,
        name,
        move |x| {
            if x == constant_c {
                constant_k
            } else {
                x
            }
        },
    )
}

pub fn image_conjugate(input: &ImageData, name: &str) -> ImageData {
    let arr = match input.point_data().get_array(name) {
        Some(x) if x.num_components() == 2 => x,
        _ => return input.clone(),
    };
    let mut buf = [0.0f64; 2];
    let mut data = Vec::with_capacity(arr.num_tuples() * 2);
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        data.push(buf[0]);
        data.push(-buf[1]);
    }
    replace_array(input, name, data, 2)
}

pub fn image_complex_multiply(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    let arr_a = match a.point_data().get_array(name) {
        Some(x) if x.num_components() == 2 => x,
        _ => return a.clone(),
    };
    let arr_b = match b.point_data().get_array(name) {
        Some(x) if x.num_components() == 2 => x,
        _ => return a.clone(),
    };
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ba = [0.0f64; 2];
    let mut bb = [0.0f64; 2];
    let mut data = Vec::with_capacity(n * 2);
    for i in 0..n {
        arr_a.tuple_as_f64(i, &mut ba);
        arr_b.tuple_as_f64(i, &mut bb);
        data.push(ba[0] * bb[0] - ba[1] * bb[1]);
        data.push(ba[1] * bb[0] + ba[0] * bb[1]);
    }
    replace_array(a, name, data, 2)
}

fn binary_op(a: &ImageData, b: &ImageData, name: &str, f: impl Fn(f64, f64) -> f64) -> ImageData {
    let arr_a = match a.point_data().get_array(name) {
        Some(x) => x,
        _ => return a.clone(),
    };
    let arr_b = match b.point_data().get_array(name) {
        Some(x) if x.num_components() == arr_a.num_components() => x,
        _ => return a.clone(),
    };
    let num_components = arr_a.num_components();
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ba = vec![0.0f64; num_components];
    let mut bb = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr_a.tuple_as_f64(i, &mut ba);
        arr_b.tuple_as_f64(i, &mut bb);
        data.extend(ba.iter().zip(&bb).map(|(&x, &y)| f(x, y)));
    }
    replace_array(a, name, data, num_components)
}

fn unary_op(input: &ImageData, name: &str, f: impl Fn(f64) -> f64) -> ImageData {
    let arr = match input.point_data().get_array(name) {
        Some(x) => x,
        _ => return input.clone(),
    };
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(arr.num_tuples() * num_components);
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        data.extend(buf.iter().map(|&x| f(x)));
    }
    replace_array(input, name, data, num_components)
}

fn replace_array(
    input: &ImageData,
    name: &str,
    data: Vec<f64>,
    num_components: usize,
) -> ImageData {
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            name,
            data,
            num_components,
        )));
    img
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

    #[test]
    fn divide_by_zero_defaults_to_scalar_max() {
        let a = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 10.0,
        );
        let b = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let r = image_divide(&a, &b, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], f64::MAX);
    }

    #[test]
    fn complex_operations_match_vtk_formulas() {
        let a = ImageData::with_dimensions(1, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("z", vec![1.0, 2.0], 2),
        ));
        let b = ImageData::with_dimensions(1, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("z", vec![3.0, 4.0], 2),
        ));

        let c = image_complex_multiply(&a, &b, "z");
        let arr = c.point_data().get_array("z").unwrap();
        let mut buf = [0.0, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [-5.0, 10.0]);

        let conj = image_conjugate(&a, "z");
        let arr = conj.point_data().get_array("z").unwrap();
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, -2.0]);
    }
}
