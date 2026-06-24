//! Transform data arrays: remap, threshold clamp, quantize.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Remap scalar values from [old_min,old_max] to [new_min,new_max].
pub fn remap_scalar_range(
    mesh: &PolyData,
    array_name: &str,
    new_min: f64,
    new_max: f64,
) -> PolyData {
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
    let old_range = (max_v - min_v).max(1e-15);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            new_min + (buf[0] - min_v) / old_range * (new_max - new_min)
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Clamp scalar values to [min, max].
pub fn clamp_scalar(mesh: &PolyData, array_name: &str, min_val: f64, max_val: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0].clamp(min_val, max_val)
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Quantize scalar values to N discrete levels.
pub fn quantize_scalar(mesh: &PolyData, array_name: &str, n_levels: usize) -> PolyData {
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
    let range = (max_v - min_v).max(1e-15);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let t = ((buf[0] - min_v) / range * n_levels as f64).floor() as usize;
            min_v
                + (t.min(n_levels - 1)) as f64 * range / n_levels as f64
                + range / (2 * n_levels) as f64
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Apply a piecewise linear transfer function to a scalar array.
pub fn apply_transfer_function(
    mesh: &PolyData,
    array_name: &str,
    control_points: &[(f64, f64)],
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let x = buf[0];
            if control_points.is_empty() {
                return x;
            }
            if x <= control_points[0].0 {
                return control_points[0].1;
            }
            if x >= control_points.last().unwrap().0 {
                return control_points.last().unwrap().1;
            }
            for w in control_points.windows(2) {
                if x >= w[0].0 && x <= w[1].0 {
                    let t = (x - w[0].0) / (w[1].0 - w[0].0);
                    return w[0].1 + t * (w[1].1 - w[0].1);
                }
            }
            x
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remap() {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 3]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 5.0, 10.0],
                1,
            )));
        let result = remap_scalar_range(&m, "v", 0.0, 1.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01);
    }
    #[test]
    fn clamp() {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 3]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![-5.0, 5.0, 15.0],
                1,
            )));
        let result = clamp_scalar(&m, "v", 0.0, 10.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 10.0);
    }
    #[test]
    fn transfer() {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 3]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let result = apply_transfer_function(&m, "v", &[(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // peak at 0.5
    }
}
