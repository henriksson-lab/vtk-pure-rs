//! Image compositing: overlay, screen, multiply, add blend modes.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Multiply blend: result = a * b.
pub fn multiply_blend(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    blend_op(a, b, array_name, |av, bv| av * bv)
}

/// Screen blend: result = 1 - (1-a)*(1-b).
pub fn screen_blend(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    blend_op(a, b, array_name, |av, bv| 1.0 - (1.0 - av) * (1.0 - bv))
}

/// Overlay blend: combines multiply and screen.
pub fn overlay_blend(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    blend_op(a, b, array_name, |av, bv| {
        if av < 0.5 {
            2.0 * av * bv
        } else {
            1.0 - 2.0 * (1.0 - av) * (1.0 - bv)
        }
    })
}

/// Additive blend with clamping: result = min(a + b, 1).
pub fn additive_blend(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    blend_op(a, b, array_name, |av, bv| (av + bv).min(1.0))
}

/// Subtractive blend: result = max(a - b, 0).
pub fn subtractive_blend(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    blend_op(a, b, array_name, |av, bv| (av - bv).max(0.0))
}

/// Soft light blend.
pub fn soft_light_blend(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    blend_op(a, b, array_name, |av, bv| {
        if bv < 0.5 {
            av - (1.0 - 2.0 * bv) * av * (1.0 - av)
        } else {
            av + (2.0 * bv - 1.0) * (av.sqrt() - av)
        }
    })
}

/// Custom blend with arbitrary function.
pub fn custom_blend(
    a: &ImageData,
    b: &ImageData,
    array_name: &str,
    f: impl Fn(f64, f64) -> f64,
) -> ImageData {
    blend_op(a, b, array_name, f)
}

fn blend_op(
    a: &ImageData,
    b: &ImageData,
    array_name: &str,
    f: impl Fn(f64, f64) -> f64,
) -> ImageData {
    let a_arr = match a.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let b_arr = match b.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let n = a_arr.num_tuples().min(b_arr.num_tuples());
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            a_arr.tuple_as_f64(i, &mut ab);
            b_arr.tuple_as_f64(i, &mut bb);
            f(ab[0], bb[0])
        })
        .collect();
    let mut result = a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiply() {
        let a = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.5,
        );
        let b = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.8,
        );
        let result = multiply_blend(&a, &b, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.4).abs() < 0.01);
    }
    #[test]
    fn screen() {
        let a = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.5,
        );
        let b = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.5,
        );
        let result = screen_blend(&a, &b, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.75).abs() < 0.01); // 1-(1-0.5)*(1-0.5)=0.75
    }
    #[test]
    fn additive() {
        let a = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.7,
        );
        let b = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.5,
        );
        let result = additive_blend(&a, &b, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // clamped to 1
    }
}
