//! Temporal data processing filters.
//!
//! Provides filters for processing sequences of datasets over time:
//! - Temporal smoothing (running average)
//! - Critical time (when a field exceeds a threshold)
//! - Temporal statistics (min/max/mean/std over time)

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the running average of scalar point data across time steps.
///
/// Returns a PolyData with the averaged scalar arrays. The geometry comes
/// from the first time step.
pub fn temporal_smooth(time_steps: &[PolyData], array_name: &str) -> Option<PolyData> {
    if time_steps.is_empty() {
        return None;
    }

    let first = &time_steps[0];
    let n = first.points.len();

    // Accumulate scalar values
    let mut sum = vec![0.0f64; n];
    let mut count = 0usize;

    for step in time_steps {
        if let Some(AnyDataArray::F64(arr)) = step.point_data().get_array(array_name) {
            if arr.len() == n && arr.num_components() == 1 {
                for i in 0..n {
                    sum[i] += arr.tuple(i)[0];
                }
                count += 1;
            }
        } else if let Some(AnyDataArray::F32(arr)) = step.point_data().get_array(array_name) {
            if arr.len() == n && arr.num_components() == 1 {
                for i in 0..n {
                    sum[i] += arr.tuple(i)[0] as f64;
                }
                count += 1;
            }
        }
    }

    if count == 0 {
        return None;
    }

    let avg: Vec<f64> = sum.iter().map(|s| s / count as f64).collect();
    let mut result = first.clone();
    let name = format!("{}_temporal_avg", array_name);
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&name, avg, 1)));
    Some(result)
}

/// Find the time at which a scalar field first exceeds a threshold at each point.
///
/// Returns a PolyData with a "CriticalTime" scalar array. Points that never
/// exceed the threshold get the value -1.0.
pub fn critical_time(
    time_steps: &[PolyData],
    times: &[f64],
    array_name: &str,
    threshold: f64,
) -> Option<PolyData> {
    if time_steps.is_empty() || times.len() != time_steps.len() {
        return None;
    }

    let first = &time_steps[0];
    let n = first.points.len();
    let mut critical = vec![-1.0f64; n];

    for (step_idx, step) in time_steps.iter().enumerate() {
        if let Some(AnyDataArray::F64(arr)) = step.point_data().get_array(array_name) {
            if arr.len() == n && arr.num_components() == 1 {
                for i in 0..n {
                    if critical[i] < 0.0 && arr.tuple(i)[0] > threshold {
                        critical[i] = times[step_idx];
                    }
                }
            }
        }
    }

    let mut result = first.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalTime",
            critical,
            1,
        )));
    Some(result)
}

/// Compute temporal statistics (min, max, mean, std) at each point across time.
///
/// Adds arrays: `{name}_min`, `{name}_max`, `{name}_mean`, `{name}_std`.
pub fn temporal_statistics(time_steps: &[PolyData], array_name: &str) -> Option<PolyData> {
    if time_steps.is_empty() {
        return None;
    }

    let first = &time_steps[0];
    let n = first.points.len();

    let mut min_vals = vec![f64::MAX; n];
    let mut max_vals = vec![f64::MIN; n];
    let mut sum = vec![0.0f64; n];
    let mut sum_sq = vec![0.0f64; n];
    let mut count = 0usize;

    for step in time_steps {
        if let Some(AnyDataArray::F64(arr)) = step.point_data().get_array(array_name) {
            if arr.len() == n && arr.num_components() == 1 {
                for i in 0..n {
                    let v = arr.tuple(i)[0];
                    min_vals[i] = min_vals[i].min(v);
                    max_vals[i] = max_vals[i].max(v);
                    sum[i] += v;
                    sum_sq[i] += v * v;
                }
                count += 1;
            }
        }
    }

    if count == 0 {
        return None;
    }

    let cf = count as f64;
    let mean: Vec<f64> = sum.iter().map(|s| s / cf).collect();
    let std_dev: Vec<f64> = sum_sq
        .iter()
        .zip(sum.iter())
        .map(|(sq, s)| {
            let m = s / cf;
            (sq / cf - m * m).max(0.0).sqrt()
        })
        .collect();

    let mut result = first.clone();
    let pd = result.point_data_mut();
    pd.add_array(AnyDataArray::F64(DataArray::from_vec(
        &format!("{array_name}_min"),
        min_vals,
        1,
    )));
    pd.add_array(AnyDataArray::F64(DataArray::from_vec(
        &format!("{array_name}_max"),
        max_vals,
        1,
    )));
    pd.add_array(AnyDataArray::F64(DataArray::from_vec(
        &format!("{array_name}_mean"),
        mean,
        1,
    )));
    pd.add_array(AnyDataArray::F64(DataArray::from_vec(
        &format!("{array_name}_std"),
        std_dev,
        1,
    )));
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    fn make_step(values: Vec<f64>) -> PolyData {
        let n = values.len();
        let pts: Vec<[f64; 3]> = (0..n).map(|i| [i as f64, 0.0, 0.0]).collect();
        let mut pd = PolyData::from_triangles(pts, vec![]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("temp", values, 1)));
        pd
    }

    #[test]
    fn smooth_two_steps() {
        let steps = vec![
            make_step(vec![0.0, 2.0, 4.0]),
            make_step(vec![2.0, 4.0, 6.0]),
        ];
        let result = temporal_smooth(&steps, "temp").unwrap();
        let avg = result.point_data().get_array("temp_temporal_avg").unwrap();
        if let AnyDataArray::F64(arr) = avg {
            assert!((arr.tuple(0)[0] - 1.0).abs() < 1e-10);
            assert!((arr.tuple(1)[0] - 3.0).abs() < 1e-10);
            assert!((arr.tuple(2)[0] - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn critical_time_test() {
        let steps = vec![
            make_step(vec![0.0, 0.5, 2.0]),
            make_step(vec![0.5, 1.5, 3.0]),
            make_step(vec![2.0, 2.5, 4.0]),
        ];
        let times = vec![0.0, 1.0, 2.0];
        let result = critical_time(&steps, &times, "temp", 1.0).unwrap();
        let ct = result.point_data().get_array("CriticalTime").unwrap();
        if let AnyDataArray::F64(arr) = ct {
            assert!(
                (arr.tuple(0)[0] - 2.0).abs() < 1e-10,
                "point 0 exceeds at t=2"
            );
            assert!(
                (arr.tuple(1)[0] - 1.0).abs() < 1e-10,
                "point 1 exceeds at t=1"
            );
            assert!(
                (arr.tuple(2)[0] - 0.0).abs() < 1e-10,
                "point 2 exceeds at t=0"
            );
        }
    }

    #[test]
    fn temporal_stats() {
        let steps = vec![
            make_step(vec![1.0, 2.0]),
            make_step(vec![3.0, 4.0]),
            make_step(vec![5.0, 6.0]),
        ];
        let result = temporal_statistics(&steps, "temp").unwrap();
        let mean = result.point_data().get_array("temp_mean").unwrap();
        if let AnyDataArray::F64(arr) = mean {
            assert!((arr.tuple(0)[0] - 3.0).abs() < 1e-10);
            assert!((arr.tuple(1)[0] - 4.0).abs() < 1e-10);
        }
    }
}
