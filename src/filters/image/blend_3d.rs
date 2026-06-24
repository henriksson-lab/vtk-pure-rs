//! 3D image blending and compositing operations.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Alpha blend two ImageData volumes.
///
/// result = alpha * a + (1-alpha) * b
pub fn blend_images(a: &ImageData, b: &ImageData, array_name: &str, alpha: f64) -> ImageData {
    let a_arr = match a.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let b_arr = match b.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let n = a_arr.num_tuples().min(b_arr.num_tuples());

    let mut output = Vec::with_capacity(n);
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    for i in 0..n {
        a_arr.tuple_as_f64(i, &mut ab);
        b_arr.tuple_as_f64(i, &mut bb);
        output.push(alpha * ab[0] + (1.0 - alpha) * bb[0]);
    }
    let mut result = a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

/// Weighted average of multiple ImageData volumes.
pub fn weighted_average_images(
    images: &[&ImageData],
    weights: &[f64],
    array_name: &str,
) -> Option<ImageData> {
    if images.is_empty() || weights.len() != images.len() {
        return None;
    }
    let first = images[0];
    let n = first.point_data().get_array(array_name)?.num_tuples();
    let w_sum: f64 = weights.iter().sum();
    if w_sum < 1e-15 {
        return None;
    }

    let mut output = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (img, &w) in images.iter().zip(weights) {
        if let Some(arr) = img.point_data().get_array(array_name) {
            for i in 0..n.min(arr.num_tuples()) {
                arr.tuple_as_f64(i, &mut buf);
                output[i] += w * buf[0] / w_sum;
            }
        }
    }
    let mut result = first.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    Some(result)
}

/// Maximum intensity projection across multiple volumes.
pub fn max_intensity_composite(images: &[&ImageData], array_name: &str) -> Option<ImageData> {
    if images.is_empty() {
        return None;
    }
    let n = images[0].point_data().get_array(array_name)?.num_tuples();
    let mut output = vec![f64::MIN; n];
    let mut buf = [0.0f64];
    for img in images {
        if let Some(arr) = img.point_data().get_array(array_name) {
            for i in 0..n.min(arr.num_tuples()) {
                arr.tuple_as_f64(i, &mut buf);
                output[i] = output[i].max(buf[0]);
            }
        }
    }
    let mut result = images[0].clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    Some(result)
}

/// Difference between two images: |a - b|.
pub fn image_abs_difference(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    let a_arr = match a.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let b_arr = match b.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let n = a_arr.num_tuples().min(b_arr.num_tuples());
    let mut output = Vec::with_capacity(n);
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    for i in 0..n {
        a_arr.tuple_as_f64(i, &mut ab);
        b_arr.tuple_as_f64(i, &mut bb);
        output.push((ab[0] - bb[0]).abs());
    }
    let mut result = a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Difference",
            output,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blend() {
        let a = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        let b = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let result = blend_images(&a, &b, "v", 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 0.01);
    }
    #[test]
    fn weighted_avg() {
        let a = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 2.0,
        );
        let b = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 4.0,
        );
        let result = weighted_average_images(&[&a, &b], &[1.0, 1.0], "v").unwrap();
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 0.01);
    }
    #[test]
    fn max_composite() {
        let a = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let b = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, y, _| y,
        );
        let result = max_intensity_composite(&[&a, &b], "v").unwrap();
        assert!(result.point_data().get_array("v").is_some());
    }
    #[test]
    fn abs_diff() {
        let a = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 5.0,
        );
        let b = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 3.0,
        );
        let result = image_abs_difference(&a, &b, "v");
        let arr = result.point_data().get_array("Difference").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 0.01);
    }
}
