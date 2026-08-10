//! Cosine of pixel values

// VTK_COS of `vtkImageMathematics` (VTK/Imaging/Math/vtkImageMathematics.cxx:221);
// the single implementation lives in `image_math_ops`.
pub use crate::filters::image::image_math_ops::image_cos;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, ImageData};
    #[test]
    fn test_image_cos() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_cos(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn test_image_cos_multi_component() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec(
                "v",
                vec![0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI, 2.0],
                2,
            ),
        ));

        let r = image_cos(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0; 2];
        arr.tuple_as_f64(0, &mut tuple);
        assert_eq!(arr.num_components(), 2);
        assert!((tuple[0] - 1.0).abs() < 1e-12);
        assert!(tuple[1].abs() < 1e-12);
        arr.tuple_as_f64(1, &mut tuple);
        assert!((tuple[0] + 1.0).abs() < 1e-12);
        assert!((tuple[1] - 2.0f64.cos()).abs() < 1e-12);
    }
}
