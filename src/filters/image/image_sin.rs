//! Sine of pixel values

// VTK_SIN of `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx:218);
// the single implementation lives in `image_math_ops`.
pub use crate::filters::image::image_math_ops::image_sin;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ImageData;
    #[test]
    fn test_image_sin() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_sin(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
