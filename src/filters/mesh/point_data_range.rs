use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the range (min, max) of a named scalar array on a PolyData.
///
/// Searches point data first, then cell data. Returns `None` if the array
/// is not found or has zero tuples.
pub fn scalar_range(input: &PolyData, array_name: &str) -> Option<(f64, f64)> {
    let arr = input
        .point_data()
        .get_array(array_name)
        .or_else(|| input.cell_data().get_array(array_name))?;

    let n: usize = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    arr.tuple_as_f64(0, &mut buf);
    let mut min_val: f64 = buf[0];
    let mut max_val: f64 = buf[0];

    for i in 1..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] < min_val {
            min_val = buf[0];
        }
        if buf[0] > max_val {
            max_val = buf[0];
        }
    }

    Some((min_val, max_val))
}

/// Normalize a named scalar array to [0, 1] range.
///
/// Reads the named array from point data, computes (value - min) / (max - min),
/// and stores the result as a new array named `{array_name}_normalized`.
/// If the array is not found or the range is zero, returns a clone.
pub fn normalize_scalar(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }

    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    let mut min_val: f64 = f64::MAX;
    let mut max_val: f64 = f64::MIN;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
        if buf[0] < min_val {
            min_val = buf[0];
        }
        if buf[0] > max_val {
            max_val = buf[0];
        }
    }

    let range: f64 = (max_val - min_val).max(1e-15);
    let normalized: Vec<f64> = values.iter().map(|v| (v - min_val) / range).collect();

    let mut pd = input.clone();
    let name = format!("{}_normalized", array_name);
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&name, normalized, 1)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CellArray, Points};

    fn make_triangle_with_scalars() -> PolyData {
        let mut points = Points::<f64>::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.5, 1.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;

        let scalars = vec![2.0f64, 8.0, 5.0];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Temperature",
                scalars,
                1,
            )));
        pd
    }

    #[test]
    fn test_scalar_range_basic() {
        let pd = make_triangle_with_scalars();
        let result = scalar_range(&pd, "Temperature");
        assert!(result.is_some());
        let (min_val, max_val) = result.unwrap();
        assert!((min_val - 2.0).abs() < 1e-10);
        assert!((max_val - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_scalar_range_missing_array() {
        let pd = make_triangle_with_scalars();
        let result = scalar_range(&pd, "NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_scalar() {
        let pd = make_triangle_with_scalars();
        let result = normalize_scalar(&pd, "Temperature");
        let arr = result
            .point_data()
            .get_array("Temperature_normalized")
            .unwrap();
        assert_eq!(arr.num_tuples(), 3);

        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10); // min -> 0.0

        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10); // max -> 1.0

        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10); // mid -> 0.5
    }
}
