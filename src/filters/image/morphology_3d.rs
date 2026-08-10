//! 3D morphological operations on ImageData: dilate, erode, open, close.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// 3D binary dilation with an ellipsoidal structuring element.
pub fn dilate_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    image_dilate_erode_3d(image, array_name, 1.0, 0.0, kernel_size_from_radius(radius))
}

/// 3D binary erosion with an ellipsoidal structuring element.
pub fn erode_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    image_dilate_erode_3d(image, array_name, 0.0, 1.0, kernel_size_from_radius(radius))
}

/// 3D morphological opening (erode then dilate).
pub fn open_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let eroded = erode_3d(image, array_name, radius);
    dilate_3d(&eroded, array_name, radius)
}

/// 3D morphological closing (dilate then erode).
pub fn close_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let dilated = dilate_3d(image, array_name, radius);
    erode_3d(&dilated, array_name, radius)
}

/// 3D morphological gradient (dilate - erode).
pub fn morphological_gradient_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let dilated = dilate_3d(image, array_name, radius);
    let eroded = erode_3d(image, array_name, radius);

    let d_arr = match dilated.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let e_arr = match eroded.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    if d_arr.num_components() != 1 || e_arr.num_components() != 1 {
        return image.clone();
    }
    let n = d_arr.num_tuples();

    let mut grad = Vec::with_capacity(n);
    let mut db = [0.0f64];
    let mut eb = [0.0f64];
    for i in 0..n {
        d_arr.tuple_as_f64(i, &mut db);
        e_arr.tuple_as_f64(i, &mut eb);
        grad.push(db[0] - eb[0]);
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorphGradient",
            grad,
            1,
        )));
    result
}

/// Dilate one value and erode another, following `vtkImageDilateErode3D`.
///
/// Re-exported from [`crate::filters::image::erode_dilate_binary`], which holds
/// the single implementation.
pub use crate::filters::image::erode_dilate_binary::image_dilate_erode_3d;

fn kernel_size_from_radius(radius: usize) -> [usize; 3] {
    let size = radius.saturating_mul(2).saturating_add(1);
    [size, size, size]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sphere_image() -> ImageData {
        ImageData::from_function(
            [20, 20, 20],
            [0.1, 0.1, 0.1],
            [0.0, 0.0, 0.0],
            "mask",
            |x, y, z| {
                if (x - 1.0).powi(2) + (y - 1.0).powi(2) + (z - 1.0).powi(2) < 0.25 {
                    1.0
                } else {
                    0.0
                }
            },
        )
    }

    #[test]
    fn dilate_grows() {
        let img = make_sphere_image();
        let dilated = dilate_3d(&img, "mask", 1);
        let orig_count = count_above(&img, "mask", 0.5);
        let new_count = count_above(&dilated, "mask", 0.5);
        assert!(new_count > orig_count);
    }

    #[test]
    fn erode_shrinks() {
        let img = make_sphere_image();
        let eroded = erode_3d(&img, "mask", 1);
        let orig_count = count_above(&img, "mask", 0.5);
        let new_count = count_above(&eroded, "mask", 0.5);
        assert!(new_count < orig_count);
    }

    #[test]
    fn open_close() {
        let img = make_sphere_image();
        let opened = open_3d(&img, "mask", 1);
        let closed = close_3d(&img, "mask", 1);
        assert!(count_above(&opened, "mask", 0.5) <= count_above(&img, "mask", 0.5));
        assert!(count_above(&closed, "mask", 0.5) >= count_above(&img, "mask", 0.5));
    }

    #[test]
    fn vtk_style_copies_unmatched_values() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![1.0, 0.0, 0.25],
                1,
            )));

        let dilated = dilate_3d(&img, "mask", 1);
        let arr = dilated.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.25);
    }

    #[test]
    fn vtk_style_processes_components_independently() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![
                    1.0, 0.0, //
                    0.0, 1.0, //
                    0.0, 0.0,
                ],
                2,
            )));

        let dilated = dilate_3d(&img, "mask", 1);
        let arr = dilated.point_data().get_array("mask").unwrap();
        assert_eq!(arr.num_components(), 2);

        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [1.0, 1.0]);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [0.0, 1.0]);
    }

    #[test]
    fn gradient_missing_array_returns_input() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = morphological_gradient_3d(&img, "missing", 1);
        assert_eq!(result.dimensions(), img.dimensions());
        assert!(result.point_data().get_array("MorphGradient").is_none());
    }

    fn count_above(img: &ImageData, name: &str, thresh: f64) -> usize {
        let arr = img.point_data().get_array(name).unwrap();
        let mut c = 0;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= thresh {
                c += 1;
            }
        }
        c
    }
}
