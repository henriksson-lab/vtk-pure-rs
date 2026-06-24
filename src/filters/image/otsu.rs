use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Otsu's optimal threshold for a scalar ImageData field.
///
/// Finds the threshold that minimizes intra-class variance of the
/// foreground/background split. Returns the threshold value.
pub fn otsu_threshold(input: &ImageData, scalars: &str, n_bins: usize) -> Option<f64> {
    let arr = input.point_data().get_array(scalars)?;
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let n_bins = n_bins.max(2);
    let mut buf = [0.0f64];

    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }

    let range = (max_v - min_v).max(1e-15);
    let bw = range / n_bins as f64;

    // Build histogram
    let mut hist = vec![0usize; n_bins];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let bin = ((buf[0] - min_v) / bw).floor() as usize;
        hist[bin.min(n_bins - 1)] += 1;
    }

    // Otsu's method
    let total = n as f64;
    let mut sum_total = 0.0;
    for (i, &c) in hist.iter().enumerate() {
        sum_total += i as f64 * c as f64;
    }

    let mut best_thresh = 0;
    let mut best_var = 0.0f64;
    let mut w_bg = 0.0;
    let mut sum_bg = 0.0;

    for t in 0..n_bins {
        w_bg += hist[t] as f64;
        if w_bg == 0.0 {
            continue;
        }
        let w_fg = total - w_bg;
        if w_fg == 0.0 {
            break;
        }

        sum_bg += t as f64 * hist[t] as f64;
        let mean_bg = sum_bg / w_bg;
        let mean_fg = (sum_total - sum_bg) / w_fg;
        let var = w_bg * w_fg * (mean_bg - mean_fg) * (mean_bg - mean_fg);

        if var > best_var {
            best_var = var;
            best_thresh = t;
        }
    }

    Some(min_v + (best_thresh as f64 + 0.5) * bw)
}

/// Apply Otsu's threshold to create a binary mask.
pub fn image_otsu(input: &ImageData, scalars: &str, n_bins: usize) -> ImageData {
    let threshold = match otsu_threshold(input, scalars, n_bins) {
        Some(t) => t,
        None => return input.clone(),
    };

    crate::filters::image::threshold::image_binary_threshold(input, scalars, threshold, f64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bimodal_distribution() {
        let mut img = ImageData::with_dimensions(20, 1, 1);
        let mut values = vec![0.0f64; 20];
        // 10 low values, 10 high values
        for i in 0..10 {
            values[i] = 10.0 + (i as f64) * 0.1;
        }
        for i in 10..20 {
            values[i] = 90.0 + (i as f64) * 0.1;
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let thresh = otsu_threshold(&img, "v", 100).unwrap();
        // Threshold should be between the two clusters
        assert!(
            thresh > 10.0 && thresh < 95.0,
            "otsu threshold = {}",
            thresh
        );
    }

    #[test]
    fn uniform_distribution() {
        let mut img = ImageData::with_dimensions(10, 1, 1);
        let values: Vec<f64> = (0..10).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let thresh = otsu_threshold(&img, "v", 10);
        assert!(thresh.is_some());
    }

    #[test]
    fn otsu_binary() {
        let mut img = ImageData::with_dimensions(10, 1, 1);
        let mut values = vec![0.0f64; 10];
        for i in 5..10 {
            values[i] = 100.0;
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_otsu(&img, "v", 50);
        assert!(result.point_data().get_array("Mask").is_some());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 1, 1);
        assert!(otsu_threshold(&img, "nope", 10).is_none());
    }
}
