//! Square of pixel values

// VTK_SQR of `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx:233);
// the single implementation lives in `image_math_ops`.
pub use crate::filters::image::image_math_ops::image_square;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ImageData;
    #[test]
    fn test_image_square() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_square(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
