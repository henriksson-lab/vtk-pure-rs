//! Mask operations on ImageData: apply, combine, invert, expand.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a binary mask to a scalar array: set masked voxels to fill_value.
pub fn apply_mask(
    image: &ImageData,
    array_name: &str,
    mask_name: &str,
    fill_value: f64,
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let mask = match image.point_data().get_array(mask_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let n = arr.num_tuples().min(mask.num_tuples());
    let mut ab = [0.0f64];
    let mut mb = [0.0f64];
    let mut output = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut ab);
        mask.tuple_as_f64(i, &mut mb);
        output.push(if mb[0] > 0.5 { ab[0] } else { fill_value });
    }
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

/// Combine two masks with AND.
pub fn mask_and(image: &ImageData, mask_a: &str, mask_b: &str, result_name: &str) -> ImageData {
    combine_masks(image, mask_a, mask_b, result_name, |a, b| {
        if a > 0.5 && b > 0.5 {
            1.0
        } else {
            0.0
        }
    })
}

/// Combine two masks with OR.
pub fn mask_or(image: &ImageData, mask_a: &str, mask_b: &str, result_name: &str) -> ImageData {
    combine_masks(image, mask_a, mask_b, result_name, |a, b| {
        if a > 0.5 || b > 0.5 {
            1.0
        } else {
            0.0
        }
    })
}

/// Combine two masks with XOR.
pub fn mask_xor(image: &ImageData, mask_a: &str, mask_b: &str, result_name: &str) -> ImageData {
    combine_masks(image, mask_a, mask_b, result_name, |a, b| {
        if (a > 0.5) != (b > 0.5) {
            1.0
        } else {
            0.0
        }
    })
}

/// Invert a mask.
pub fn mask_invert(image: &ImageData, mask_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(mask_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let mut buf = [0.0f64];
    let output: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                0.0
            } else {
                1.0
            }
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(mask_name, output, 1)));
    result
}

/// Count foreground voxels in a mask.
pub fn mask_count(image: &ImageData, mask_name: &str) -> usize {
    let arr = match image.point_data().get_array(mask_name) {
        Some(a) => a,
        None => return 0,
    };
    let mut buf = [0.0f64];
    let mut count = 0;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] > 0.5 {
            count += 1;
        }
    }
    count
}

/// Compute Dice coefficient between two masks: 2*|A∩B|/(|A|+|B|).
pub fn dice_coefficient(image: &ImageData, mask_a: &str, mask_b: &str) -> f64 {
    let a = match image.point_data().get_array(mask_a) {
        Some(x) => x,
        None => return 0.0,
    };
    let b = match image.point_data().get_array(mask_b) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = a.num_tuples().min(b.num_tuples());
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    let mut both = 0usize;
    let mut count_a = 0usize;
    let mut count_b = 0usize;
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        if ab[0] > 0.5 {
            count_a += 1;
        }
        if bb[0] > 0.5 {
            count_b += 1;
        }
        if ab[0] > 0.5 && bb[0] > 0.5 {
            both += 1;
        }
    }
    let denom = count_a + count_b;
    if denom > 0 {
        2.0 * both as f64 / denom as f64
    } else {
        0.0
    }
}

fn combine_masks(
    image: &ImageData,
    a: &str,
    b: &str,
    result_name: &str,
    op: impl Fn(f64, f64) -> f64,
) -> ImageData {
    let arr_a = match image.point_data().get_array(a) {
        Some(x) => x,
        None => return image.clone(),
    };
    let arr_b = match image.point_data().get_array(b) {
        Some(x) => x,
        None => return image.clone(),
    };
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    let output: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut ab);
            arr_b.tuple_as_f64(i, &mut bb);
            op(ab[0], bb[0])
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            output,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_apply() {
        let mut img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let mask: Vec<f64> = (0..25).map(|i| if i < 12 { 1.0 } else { 0.0 }).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", mask, 1)));
        let result = apply_mask(&img, "v", "mask", -1.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(20, &mut buf);
        assert_eq!(buf[0], -1.0); // masked
    }
    #[test]
    fn mask_logic() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 1.0, 0.0, 0.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![1.0, 0.0, 1.0, 0.0],
                1,
            )));
        let result = mask_and(&img, "a", "b", "c");
        assert_eq!(mask_count(&result, "c"), 1); // only first voxel
    }
    #[test]
    fn dice() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 1.0, 0.0, 0.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![1.0, 1.0, 1.0, 0.0],
                1,
            )));
        let d = dice_coefficient(&img, "a", "b");
        assert!((d - 0.8).abs() < 0.01); // 2*2/(2+3)=0.8
    }
}
