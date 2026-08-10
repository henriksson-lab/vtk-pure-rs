//! Image arithmetic: element-wise add, subtract, multiply, divide, max, min.

use crate::data::{AnyDataArray, DataArray, ImageData};

// The element-wise binary operations and the unary ABS/SQRT/EXP operations are
// `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx); the single
// implementation lives in `image_math_ops`, which follows the C++ in operating on
// every scalar component of the tuple.
pub use crate::filters::image::image_math_ops::{
    image_abs, image_add, image_divide, image_exp, image_max, image_min, image_multiply,
    image_sqrt, image_subtract,
};
// `image_scale` is VTK_MULTIPLYBYK; the single implementation is reached via `math`.
pub use crate::filters::image::math::image_scale;

/// Natural logarithm (VTK_LOG of `vtkImageMathematics`).
///
/// Alias for [`crate::filters::image::image_math_ops::image_log`]; kept as a wrapper
/// rather than a re-export so both spellings remain available from this module.
pub fn image_ln(image: &ImageData, name: &str) -> ImageData {
    crate::filters::image::image_math_ops::image_log(image, name)
}

/// Add a constant to an array.
pub fn image_offset(image: &ImageData, name: &str, offset: f64) -> ImageData {
    unary_arith(image, name, |x| x + offset)
}

fn unary_arith(img: &ImageData, name: &str, f: impl Fn(f64) -> f64) -> ImageData {
    let arr = match img.point_data().get_array(name) {
        Some(x) => x,
        None => return img.clone(),
    };
    let mut buf = [0.0f64];
    let d: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            f(buf[0])
        })
        .collect();
    let mut r = img.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, d, 1)));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    /// VTK_LOG is a plain `std::log`, so negative inputs are NaN and zero is -inf
    /// (the old guarded `image_ln` clamped to `1e-30` instead).
    #[test]
    fn ln_is_unguarded_natural_log() {
        let img = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = image_ln(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].is_infinite() && buf[0].is_sign_negative()); // ln(0)
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0); // ln(1)
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 2.0f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn offset_adds_constant() {
        let img = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 4.0,
        );
        let r = image_offset(&img, "v", -1.5);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.5).abs() < 1e-12);
    }
}
