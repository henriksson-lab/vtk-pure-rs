//! Multi-level Otsu thresholding.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute optimal Otsu threshold for a scalar image.
pub fn otsu_threshold_value(input: &ImageData, scalars: &str, bins: usize) -> f64 {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0.0,
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let bins = bins.max(2);

    let mut hist = vec![0usize; bins];
    for &v in &vals {
        let bi = (((v - mn) / range * bins as f64).floor() as usize).min(bins - 1);
        hist[bi] += 1;
    }

    let total = n as f64;
    let mut sum_total = 0.0;
    for i in 0..bins {
        sum_total += i as f64 * hist[i] as f64;
    }

    let mut best_thresh = 0;
    let mut best_var = 0.0f64;
    let mut w0 = 0.0;
    let mut sum0 = 0.0;

    for t in 0..bins {
        w0 += hist[t] as f64;
        if w0 == 0.0 {
            continue;
        }
        let w1 = total - w0;
        if w1 == 0.0 {
            break;
        }
        sum0 += t as f64 * hist[t] as f64;
        let m0 = sum0 / w0;
        let m1 = (sum_total - sum0) / w1;
        let var = w0 * w1 * (m0 - m1) * (m0 - m1);
        if var > best_var {
            best_var = var;
            best_thresh = t;
        }
    }

    mn + (best_thresh as f64 + 0.5) / bins as f64 * range
}

/// Apply Otsu threshold and return binary image.
pub fn otsu_binary(input: &ImageData, scalars: &str, bins: usize) -> ImageData {
    let thresh = otsu_threshold_value(input, scalars, bins);
    let arr = input.point_data().get_array(scalars).unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= thresh {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Multi-level Otsu: recursively split into N classes.
pub fn otsu_multi_level(input: &ImageData, scalars: &str, levels: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut thresholds = Vec::new();
    find_thresholds_recursive(&vals, levels.max(2) - 1, &mut thresholds);
    thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let data: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let mut level = 0;
            for &t in &thresholds {
                if v >= t {
                    level += 1;
                }
            }
            level as f64
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Levels", data, 1)))
}

fn find_thresholds_recursive(vals: &[f64], remaining: usize, thresholds: &mut Vec<f64>) {
    if remaining == 0 || vals.len() < 2 {
        return;
    }
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mid = (mn + mx) / 2.0;
    thresholds.push(mid);
    if remaining > 1 {
        let lo: Vec<f64> = vals.iter().filter(|&&v| v < mid).cloned().collect();
        let hi: Vec<f64> = vals.iter().filter(|&&v| v >= mid).cloned().collect();
        find_thresholds_recursive(&lo, remaining / 2, thresholds);
        find_thresholds_recursive(&hi, remaining - remaining / 2 - 1, thresholds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_otsu() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| if x < 10.0 { 20.0 } else { 200.0 },
        );
        let t = otsu_threshold_value(&img, "v", 256);
        assert!(t > 10.0 && t < 210.0, "otsu threshold = {t}");
    }
    #[test]
    fn test_binary() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| if x < 5.0 { 0.0 } else { 255.0 },
        );
        let r = otsu_binary(&img, "v", 256);
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
    #[test]
    fn test_multi() {
        let img = ImageData::from_function(
            [12, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * 20.0,
        );
        let r = otsu_multi_level(&img, "v", 3);
        assert!(r.point_data().get_array("Levels").is_some());
    }
}
