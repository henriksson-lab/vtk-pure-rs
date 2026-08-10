//! Arctangent of pixel values

// VTK_ATAN of `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx:239);
// the single implementation lives in `image_math_ops`.
pub use crate::filters::image::image_math_ops::image_atan;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ImageData;
    #[test]
    fn test_image_atan() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_atan(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
