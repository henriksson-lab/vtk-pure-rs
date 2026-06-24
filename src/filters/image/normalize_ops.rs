//! Image normalization operations: min-max, z-score, percentile, histogram match.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Normalize array values to [0, 1] range.
pub fn normalize_min_max(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0] - min_v) / range
        })
        .collect();
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}

/// Z-score normalization: (x - mean) / std.
pub fn normalize_zscore(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        sum += buf[0];
        sum2 += buf[0] * buf[0];
    }
    let mean = sum / n as f64;
    let std = ((sum2 / n as f64) - mean * mean).max(0.0).sqrt().max(1e-15);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0] - mean) / std
        })
        .collect();
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}

/// Percentile normalization: map P-th to Q-th percentile to [0, 1].
pub fn normalize_percentile(
    image: &ImageData,
    array_name: &str,
    low_pct: f64,
    high_pct: f64,
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo = sorted[(low_pct * (n - 1) as f64) as usize];
    let hi = sorted[(high_pct * (n - 1) as f64) as usize];
    let range = (hi - lo).max(1e-15);
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| ((v - lo) / range).clamp(0.0, 1.0))
        .collect();
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}

/// Rescale to a specific output range [out_min, out_max].
pub fn rescale_to_range(
    image: &ImageData,
    array_name: &str,
    out_min: f64,
    out_max: f64,
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            out_min + (buf[0] - min_v) / range * (out_max - out_min)
        })
        .collect();
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn min_max() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * 10.0,
        );
        let r = normalize_min_max(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0]).abs() < 0.01);
        arr.tuple_as_f64(9, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01);
    }
    #[test]
    fn zscore() {
        let img = ImageData::from_function(
            [100, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = normalize_zscore(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        // Mean should be ~0
        let mut sum = 0.0;
        let mut buf = [0.0f64];
        for i in 0..100 {
            arr.tuple_as_f64(i, &mut buf);
            sum += buf[0];
        }
        assert!((sum / 100.0).abs() < 0.1);
    }
    #[test]
    fn rescale() {
        let img = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = rescale_to_range(&img, "v", 10.0, 20.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 10.0).abs() < 0.01);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 20.0).abs() < 0.01);
    }
}
