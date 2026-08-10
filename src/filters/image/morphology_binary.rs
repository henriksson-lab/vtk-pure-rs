//! Binary morphological operations (structuring element based).

use crate::data::ImageData;
use crate::filters::image::erode_dilate_binary::image_dilate_erode_3d;

/// Binary dilation with a square structuring element.
pub fn binary_dilate(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    morph_op(input, scalars, radius, true)
}

/// Binary erosion with a square structuring element.
pub fn binary_erode(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    morph_op(input, scalars, radius, false)
}

/// Binary opening (erode then dilate).
pub fn binary_open(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    binary_dilate(&binary_erode(input, scalars, radius), scalars, radius)
}

/// Binary closing (dilate then erode).
pub fn binary_close(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    binary_erode(&binary_dilate(input, scalars, radius), scalars, radius)
}

fn morph_op(input: &ImageData, scalars: &str, radius: usize, dilate: bool) -> ImageData {
    let (dilate_value, erode_value) = if dilate { (1.0, 0.0) } else { (0.0, 1.0) };
    image_dilate_erode_3d(
        input,
        scalars,
        dilate_value,
        erode_value,
        kernel_size_from_radius(radius),
    )
}

fn kernel_size_from_radius(radius: usize) -> [usize; 3] {
    let size = radius.saturating_mul(2).saturating_add(1);
    [size, size, size]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_dilate() {
        let img = ImageData::from_function(
            [7, 7, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 3.0).abs() < 0.5 && (y - 3.0).abs() < 0.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = binary_dilate(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(4 + 3 * 7, &mut buf); // neighbor of center
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn test_erode() {
        let img = ImageData::from_function(
            [7, 7, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 3.0).abs() < 1.5 && (y - 3.0).abs() < 1.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = binary_erode(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(3 + 3 * 7, &mut buf);
        assert_eq!(buf[0], 1.0); // center survives
    }
    #[test]
    fn test_open_close() {
        let img = ImageData::from_function(
            [9, 9, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 4.0).abs() < 2.5 && (y - 4.0).abs() < 2.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let opened = binary_open(&img, "v", 1);
        let closed = binary_close(&img, "v", 1);
        assert_eq!(opened.dimensions(), [9, 9, 1]);
        assert_eq!(closed.dimensions(), [9, 9, 1]);
    }

    #[test]
    fn test_non_binary_values_are_copied() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.0, 0.25],
                1,
            )));

        let result = binary_dilate(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.25);
    }

    #[test]
    fn test_preserves_other_point_arrays() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.0, 0.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "other",
                vec![3.0, 4.0, 5.0],
                1,
            )));

        let result = binary_dilate(&img, "v", 1);
        assert!(result.point_data().get_array("other").is_some());
    }
}
