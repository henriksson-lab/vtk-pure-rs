//! Photoshop-style blend modes for images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Screen blend: 1 - (1-a)(1-b). Values assumed in [0,1].
pub fn blend_screen(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    blend_op(a, b, name, |x, y| 1.0 - (1.0 - x) * (1.0 - y))
}

/// Overlay blend: combines multiply and screen.
pub fn blend_overlay(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    blend_op(a, b, name, |x, y| {
        if x < 0.5 {
            2.0 * x * y
        } else {
            1.0 - 2.0 * (1.0 - x) * (1.0 - y)
        }
    })
}

/// Soft light blend.
pub fn blend_soft_light(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    blend_op(a, b, name, |x, y| {
        if y < 0.5 {
            x - (1.0 - 2.0 * y) * x * (1.0 - x)
        } else {
            x + (2.0 * y - 1.0) * (x.sqrt() - x)
        }
    })
}

/// Hard light blend.
pub fn blend_hard_light(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    blend_op(a, b, name, |x, y| {
        if y < 0.5 {
            2.0 * x * y
        } else {
            1.0 - 2.0 * (1.0 - x) * (1.0 - y)
        }
    })
}

/// Difference blend: |a - b|.
pub fn blend_difference(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    blend_op(a, b, name, |x, y| (x - y).abs())
}

/// Multiply blend: a * b.
pub fn blend_multiply(a: &ImageData, b: &ImageData, name: &str) -> ImageData {
    blend_op(a, b, name, |x, y| x * y)
}

fn blend_op(a: &ImageData, b: &ImageData, name: &str, f: impl Fn(f64, f64) -> f64) -> ImageData {
    let aa = match a.point_data().get_array(name) {
        Some(x) if x.num_components() == 1 => x,
        _ => return a.clone(),
    };
    let bb = match b.point_data().get_array(name) {
        Some(x) if x.num_components() == 1 => x,
        _ => return a.clone(),
    };
    let n = aa.num_tuples().min(bb.num_tuples());
    let mut ba = [0.0f64];
    let mut bb2 = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            aa.tuple_as_f64(i, &mut ba);
            bb.tuple_as_f64(i, &mut bb2);
            f(ba[0], bb2[0])
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
    fn test_screen() {
        let a = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.5,
        );
        let b = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.5,
        );
        let r = blend_screen(&a, &b, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.75).abs() < 1e-10); // 1-(1-0.5)(1-0.5)=0.75
    }
    #[test]
    fn test_difference() {
        let a = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.8,
        );
        let b = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.3,
        );
        let r = blend_difference(&a, &b, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);
    }
}
