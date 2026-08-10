//! Posterize to 4 levels
use crate::data::ImageData;

/// Posterize to 4 levels.
///
/// Thin wrapper over [`crate::filters::image::quantize::image_posterize`], which owns the
/// single equal-width binning implementation. Levels are spread over the actual data
/// range rather than assuming the input is normalised to `[0, 1]`.
pub fn image_posterize(input: &ImageData, scalars: &str) -> ImageData {
    crate::filters::image::quantize::image_posterize(input, scalars, 4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn test_image_posterize() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_posterize(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn posterizes_to_four_levels() {
        let img = ImageData::with_dimensions(6, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-0.1, 0.0, 0.25, 0.5, 0.75, 1.0], 1),
        ));

        let values = image_posterize(&img, "v")
            .point_data()
            .get_array("v")
            .unwrap()
            .to_f64_vec();

        let mut distinct: Vec<f64> = values.clone();
        distinct.sort_by(|a, b| a.total_cmp(b));
        distinct.dedup();
        assert!(
            distinct.len() <= 4,
            "expected at most 4 levels: {distinct:?}"
        );
        // The extremes of the data range are preserved exactly.
        assert!((values[0] + 0.1).abs() < 1e-12);
        assert!((values[5] - 1.0).abs() < 1e-12);
    }
}
