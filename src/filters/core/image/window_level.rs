//! Window/level contrast adjustment for scalar ImageData (common in medical imaging).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply window/level contrast to a scalar array.
///
/// Values in [center-width/2, center+width/2] are mapped to [0, 1].
/// Values outside are clamped.
pub fn window_level(image: &ImageData, array_name: &str, center: f64, width: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let lo = center - width / 2.0;
    let hi = center + width / 2.0;
    let range = (hi - lo).max(1e-15);
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            ((buf[0] - lo) / range).clamp(0.0, 1.0)
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Auto window/level: use percentile-based window.
pub fn auto_window_level(
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
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let lo_idx = (low_pct * (n - 1) as f64) as usize;
    let hi_idx = (high_pct * (n - 1) as f64) as usize;
    let lo = values[lo_idx.min(n - 1)];
    let hi = values[hi_idx.min(n - 1)];
    let center = (lo + hi) / 2.0;
    let width = (hi - lo).max(1e-15);
    window_level(image, array_name, center, width)
}

/// Gamma correction on a scalar array.
pub fn gamma_correction(image: &ImageData, array_name: &str, gamma: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let t = ((buf[0] - min_v) / range).clamp(0.0, 1.0);
            min_v + t.powf(gamma) * range
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Sigmoid contrast enhancement.
pub fn sigmoid_contrast(image: &ImageData, array_name: &str, alpha: f64, beta: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            1.0 / (1.0 + (-alpha * (buf[0] - beta)).exp())
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * 10.0,
        );
        let result = window_level(&img, "v", 50.0, 40.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] <= 0.01); // below window
    }
    #[test]
    fn auto_wl() {
        let img = ImageData::from_function(
            [100, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = auto_window_level(&img, "v", 0.05, 0.95);
        assert!(result.point_data().get_array("v").is_some());
    }
    #[test]
    fn gamma() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x / 9.0,
        );
        let result = gamma_correction(&img, "v", 2.0);
        assert!(result.point_data().get_array("v").is_some());
    }
    #[test]
    fn sigmoid() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = sigmoid_contrast(&img, "v", 1.0, 5.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(5, &mut buf);
        assert!((buf[0] - 0.5).abs() < 0.01); // sigmoid midpoint at beta=5
    }
}
