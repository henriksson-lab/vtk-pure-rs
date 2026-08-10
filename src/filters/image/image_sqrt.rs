//! Square root of pixel values

// VTK_SQRT of `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx:236);
// the single implementation lives in `image_math_ops`.
pub use crate::filters::image::image_math_ops::image_sqrt;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ImageData;
    #[test]
    fn test_image_sqrt() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_sqrt(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn negative_values_follow_sqrt_domain() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| -1.0,
        );
        let r = image_sqrt(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut value = [0.0f64];
        arr.tuple_as_f64(0, &mut value);
        assert!(value[0].is_nan());
    }
}
