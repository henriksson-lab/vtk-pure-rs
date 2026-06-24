//! Advanced data array operations: normalization, standardization, binning.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Standardize a scalar array (z-score: (x - mean) / std).
pub fn standardize_array(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    if n < 2 {
        return mesh.clone();
    }

    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    let mean = values.iter().sum::<f64>() / n as f64;
    let std = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64).sqrt();

    let standardized: Vec<f64> = if std > 1e-15 {
        values.iter().map(|v| (v - mean) / std).collect()
    } else {
        vec![0.0; n]
    };

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{array_name}_zscore"),
            standardized,
            1,
        )));
    result
}

/// Min-max normalize to [0, 1].
pub fn min_max_normalize(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
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
    let range = max_v - min_v;
    let mut normalized = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        normalized.push(if range > 1e-15 {
            (buf[0] - min_v) / range
        } else {
            0.5
        });
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{array_name}_norm"),
            normalized,
            1,
        )));
    result
}

/// Bin scalar values into discrete categories.
pub fn bin_array(mesh: &PolyData, array_name: &str, n_bins: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
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
    let range = max_v - min_v;
    let mut binned = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let bin = if range > 1e-15 {
            (((buf[0] - min_v) / range * n_bins as f64) as usize).min(n_bins - 1)
        } else {
            0
        };
        binned.push(bin as f64);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{array_name}_bin"),
            binned,
            1,
        )));
    result
}

/// Compute cumulative sum of a scalar array.
pub fn cumulative_sum(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut cumsum = Vec::with_capacity(n);
    let mut total = 0.0;
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        total += buf[0];
        cumsum.push(total);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{array_name}_cumsum"),
            cumsum,
            1,
        )));
    result
}

/// Compute rolling mean with window size.
pub fn rolling_mean(mesh: &PolyData, array_name: &str, window: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    let mut rolling = Vec::with_capacity(n);
    for i in 0..n {
        let start = if i >= window { i - window + 1 } else { 0 };
        let sum: f64 = values[start..=i].iter().sum();
        rolling.push(sum / (i - start + 1) as f64);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{array_name}_rolling"),
            rolling,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh() -> PolyData {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 5]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                1,
            )));
        m
    }

    #[test]
    fn z_score() {
        let result = standardize_array(&make_mesh(), "val");
        let arr = result.point_data().get_array("val_zscore").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0].abs() < 0.01); // mean value → z ≈ 0
    }

    #[test]
    fn normalize() {
        let result = min_max_normalize(&make_mesh(), "val");
        let arr = result.point_data().get_array("val_norm").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 0.01);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn binning() {
        let result = bin_array(&make_mesh(), "val", 5);
        assert!(result.point_data().get_array("val_bin").is_some());
    }

    #[test]
    fn cumsum() {
        let result = cumulative_sum(&make_mesh(), "val");
        let arr = result.point_data().get_array("val_cumsum").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 150.0).abs() < 0.01); // 10+20+30+40+50
    }

    #[test]
    fn rolling() {
        let result = rolling_mean(&make_mesh(), "val", 3);
        assert!(result.point_data().get_array("val_rolling").is_some());
    }
}
