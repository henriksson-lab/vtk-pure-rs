//! Natural logarithm of pixel values

// VTK_LOG of `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx:227) is a
// plain `std::log`; the single implementation is `image_math_ops::image_log`, reached
// here through `arithmetic::image_ln`.
pub use crate::filters::image::arithmetic::image_ln;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ImageData;
    #[test]
    fn test_image_ln() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_ln(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
