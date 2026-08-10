use crate::data::{AnyDataArray, DataArray, ImageData};

/// Bilateral filter on ImageData: edge-preserving smoothing using both spatial
/// and intensity distance weights.
///
/// For each voxel, neighboring voxels contribute based on their spatial distance
/// (controlled by `spatial_sigma`) and intensity difference (controlled by
/// `intensity_sigma`). The kernel radius is derived from `spatial_sigma`.
///
/// The result is stored as a "BilateralFiltered" point data array on the
/// returned ImageData.
///
/// Thin wrapper: the filtering itself is the single implementation in
/// [`crate::filters::image::bilateral_denoise::bilateral_filter`], which writes
/// its result back under the input scalar name; this entry point keeps the
/// input image intact and appends the result as "BilateralFiltered".
pub fn bilateral_filter(
    input: &ImageData,
    scalars: &str,
    spatial_sigma: f64,
    intensity_sigma: f64,
) -> ImageData {
    let dims = input.dimensions();
    let n: usize = dims[0] * dims[1] * dims[2];
    match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == n => {}
        _ => return input.clone(),
    }
    if n == 0 || spatial_sigma <= 0.0 || intensity_sigma <= 0.0 {
        return input.clone();
    }

    let filtered = crate::filters::image::bilateral_denoise::bilateral_filter(
        input,
        scalars,
        spatial_sigma,
        intensity_sigma,
    );
    let result = match filtered.point_data().get_array(scalars) {
        Some(a) => a.to_f64_vec(),
        None => return input.clone(),
    };

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BilateralFiltered",
            result,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_field_unchanged() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vals",
                vec![5.0; 16],
                1,
            )));

        let result = bilateral_filter(&img, "vals", 1.0, 1.0);
        let arr = result.point_data().get_array("BilateralFiltered").unwrap();
        let mut buf = [0.0f64];
        for i in 0..16 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(
                (buf[0] - 5.0).abs() < 1e-10,
                "uniform field should be preserved"
            );
        }
    }

    #[test]
    fn preserves_step_edge() {
        let mut img = ImageData::with_dimensions(6, 1, 1);
        // Step edge: low on left, high on right
        let vals: Vec<f64> = vec![0.0, 0.0, 0.0, 100.0, 100.0, 100.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vals", vals, 1)));

        let result = bilateral_filter(&img, "vals", 1.0, 5.0);
        let arr = result.point_data().get_array("BilateralFiltered").unwrap();
        let mut buf = [0.0f64];
        // Far left should stay near 0
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 20.0, "left side should remain low: {}", buf[0]);
        // Far right should stay near 100
        arr.tuple_as_f64(5, &mut buf);
        assert!(buf[0] > 80.0, "right side should remain high: {}", buf[0]);
    }

    #[test]
    fn missing_scalars_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = bilateral_filter(&img, "nonexistent", 1.0, 1.0);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }
}
