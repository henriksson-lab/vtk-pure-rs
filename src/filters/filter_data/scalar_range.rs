use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the range (min, max) of a scalar array.
pub fn scalar_range(input: &PolyData, array_name: &str) -> Option<[f64; 2]> {
    let arr = input
        .point_data()
        .get_array(array_name)
        .or_else(|| input.cell_data().get_array(array_name))?;

    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    arr.tuple_as_f64(0, &mut buf);
    let mut min_val = buf[0];
    let mut max_val = buf[0];

    for i in 1..n {
        arr.tuple_as_f64(i, &mut buf);
        min_val = min_val.min(buf[0]);
        max_val = max_val.max(buf[0]);
    }

    Some([min_val, max_val])
}

/// Normalize a scalar array to [0, 1] range.
///
/// Adds a new array named `{array_name}_normalized` with values mapped
/// linearly from [min, max] to [0, 1].
pub fn normalize_array(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
        min_val = min_val.min(buf[0]);
        max_val = max_val.max(buf[0]);
    }

    let range = (max_val - min_val).max(1e-15);
    let normalized: Vec<f64> = values.iter().map(|v| (v - min_val) / range).collect();

    let mut pd = input.clone();
    let name = format!("{}_normalized", array_name);
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&name, normalized, 1)));
    pd
}

/// Rescale a scalar array to a new [new_min, new_max] range.
pub fn rescale_array(input: &PolyData, array_name: &str, new_min: f64, new_max: f64) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
        min_val = min_val.min(buf[0]);
        max_val = max_val.max(buf[0]);
    }

    let range = (max_val - min_val).max(1e-15);
    let new_range = new_max - new_min;
    let rescaled: Vec<f64> = values
        .iter()
        .map(|v| new_min + new_range * (v - min_val) / range)
        .collect();

    let mut pd = input.clone();
    // Replace the array in-place
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                array_name,
                rescaled.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *pd.point_data_mut() = new_attrs;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pd() -> PolyData {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                1,
            )));
        pd
    }

    #[test]
    fn range() {
        let pd = make_pd();
        let r = scalar_range(&pd, "val").unwrap();
        assert_eq!(r, [0.0, 100.0]);
    }

    #[test]
    fn range_missing() {
        let pd = make_pd();
        assert!(scalar_range(&pd, "nope").is_none());
    }

    #[test]
    fn normalize() {
        let pd = make_pd();
        let result = normalize_array(&pd, "val");
        let arr = result.point_data().get_array("val_normalized").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn rescale() {
        let pd = make_pd();
        let result = rescale_array(&pd, "val", -1.0, 1.0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - (-1.0)).abs() < 1e-10);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }
}
