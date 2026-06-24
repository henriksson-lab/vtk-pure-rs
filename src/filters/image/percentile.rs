use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute percentiles of an ImageData scalar field.
///
/// Returns the value at the given percentile (0-100).
pub fn image_percentile(input: &ImageData, scalars: &str, percentile: f64) -> Option<f64> {
    let arr = input.point_data().get_array(scalars)?;
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p = percentile.clamp(0.0, 100.0) / 100.0;
    let idx = ((n - 1) as f64 * p) as usize;
    Some(values[idx.min(n - 1)])
}

/// Clip ImageData values to percentile range [lo_pct, hi_pct].
///
/// Values below lo_pct percentile become lo_pct value, above hi_pct become hi_pct value.
pub fn image_clip_percentile(
    input: &ImageData,
    scalars: &str,
    lo_pct: f64,
    hi_pct: f64,
) -> ImageData {
    let lo_val = match image_percentile(input, scalars, lo_pct) {
        Some(v) => v,
        None => return input.clone(),
    };
    let hi_val = match image_percentile(input, scalars, hi_pct) {
        Some(v) => v,
        None => return input.clone(),
    };

    crate::filters::image::abs::image_clamp(input, scalars, lo_val, hi_val)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_img() -> ImageData {
        let mut img = ImageData::with_dimensions(10, 1, 1);
        let values: Vec<f64> = (0..10).map(|i| (i * 10) as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));
        img
    }

    #[test]
    fn median_percentile() {
        let img = make_img();
        let p50 = image_percentile(&img, "v", 50.0).unwrap();
        assert!((p50 - 40.0).abs() < 1e-10 || (p50 - 50.0).abs() < 1e-10);
    }

    #[test]
    fn min_max_percentile() {
        let img = make_img();
        let p0 = image_percentile(&img, "v", 0.0).unwrap();
        let p100 = image_percentile(&img, "v", 100.0).unwrap();
        assert_eq!(p0, 0.0);
        assert_eq!(p100, 90.0);
    }

    #[test]
    fn clip_percentile() {
        let img = make_img();
        let result = image_clip_percentile(&img, "v", 10.0, 90.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // All values should be within the percentile range
        let lo = image_percentile(&img, "v", 10.0).unwrap();
        let hi = image_percentile(&img, "v", 90.0).unwrap();
        for i in 0..10 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= lo - 1e-10 && buf[0] <= hi + 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let img = make_img();
        assert!(image_percentile(&img, "nope", 50.0).is_none());
    }
}
