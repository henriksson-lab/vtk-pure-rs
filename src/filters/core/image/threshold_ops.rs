//! Image thresholding operations: Otsu, adaptive, hysteresis, multi-level.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Hysteresis thresholding: connected to strong-threshold voxels via weak-threshold.
pub fn hysteresis_threshold(image: &ImageData, array_name: &str, low: f64, high: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut mask = vec![0u8; n]; // 0=bg, 1=weak, 2=strong
    for i in 0..n {
        if vals[i] >= high {
            mask[i] = 2;
        } else if vals[i] >= low {
            mask[i] = 1;
        }
    }

    // Flood fill from strong to weak
    let mut q = std::collections::VecDeque::new();
    for i in 0..n {
        if mask[i] == 2 {
            q.push_back(i);
        }
    }
    while let Some(vi) = q.pop_front() {
        let iz = vi / (dims[0] * dims[1]);
        let rem = vi % (dims[0] * dims[1]);
        let iy = rem / dims[0];
        let ix = rem % dims[0];
        for dz in -1i64..=1 {
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let nx = ix as i64 + dx;
                    let ny = iy as i64 + dy;
                    let nz = iz as i64 + dz;
                    if nx >= 0
                        && ny >= 0
                        && nz >= 0
                        && (nx as usize) < dims[0]
                        && (ny as usize) < dims[1]
                        && (nz as usize) < dims[2]
                    {
                        let ni =
                            nx as usize + ny as usize * dims[0] + nz as usize * dims[0] * dims[1];
                        if mask[ni] == 1 {
                            mask[ni] = 2;
                            q.push_back(ni);
                        }
                    }
                }
            }
        }
    }

    let data: Vec<f64> = mask
        .iter()
        .map(|&m| if m == 2 { 1.0 } else { 0.0 })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Multi-level threshold: assign each voxel to a level based on thresholds.
pub fn multi_level_threshold(image: &ImageData, array_name: &str, thresholds: &[f64]) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let mut level = 0;
            for &t in thresholds {
                if buf[0] >= t {
                    level += 1;
                } else {
                    break;
                }
            }
            level as f64
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ThresholdLevel",
            data,
            1,
        )));
    result
}

/// Percentile-based threshold: threshold at the P-th percentile.
pub fn percentile_threshold(image: &ImageData, array_name: &str, percentile: f64) -> ImageData {
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
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (percentile * (n - 1) as f64) as usize;
    let threshold = vals[idx.min(n - 1)];

    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                1.0
            } else {
                0.0
            }
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
    fn hysteresis() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                let r = ((x - 5.0).powi(2) + (y - 5.0).powi(2)).sqrt();
                if r < 2.0 {
                    1.0
                } else if r < 4.0 {
                    0.5
                } else {
                    0.0
                }
            },
        );
        let result = hysteresis_threshold(&img, "v", 0.3, 0.8);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        let mut count = 0;
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        assert!(count > 5); // should include both strong and connected weak
    }
    #[test]
    fn multi_level() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = multi_level_threshold(&img, "v", &[3.0, 6.0]);
        assert!(result.point_data().get_array("ThresholdLevel").is_some());
    }
    #[test]
    fn percentile() {
        let img = ImageData::from_function(
            [100, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = percentile_threshold(&img, "v", 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        let mut count = 0;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        assert!((count as f64 - 50.0).abs() < 5.0); // ~50% above median
    }
}
