use crate::data::{AnyDataArray, DataArray, PolyData};

/// Add two scalar arrays element-wise and store the result.
pub fn array_add(input: &PolyData, name_a: &str, name_b: &str, result_name: &str) -> PolyData {
    array_op(input, name_a, name_b, result_name, |a, b| a + b)
}

/// Subtract array B from array A element-wise.
pub fn array_subtract(input: &PolyData, name_a: &str, name_b: &str, result_name: &str) -> PolyData {
    array_op(input, name_a, name_b, result_name, |a, b| a - b)
}

/// Multiply two scalar arrays element-wise.
pub fn array_multiply(input: &PolyData, name_a: &str, name_b: &str, result_name: &str) -> PolyData {
    array_op(input, name_a, name_b, result_name, |a, b| a * b)
}

/// Scale a scalar array by a constant.
pub fn array_scale(input: &PolyData, name: &str, factor: f64, result_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc = arr.num_components();
    let nt = arr.num_tuples();
    let mut data = Vec::with_capacity(nt * nc);
    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        for v in &buf {
            data.push(v * factor);
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            data,
            nc,
        )));
    pd
}

/// Compute the magnitude of a vector array (3-component → 1-component).
pub fn array_magnitude(input: &PolyData, name: &str, result_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(name) {
        Some(a) if a.num_components() >= 2 => a,
        _ => return input.clone(),
    };

    let nc = arr.num_components();
    let nt = arr.num_tuples();
    let mut data = Vec::with_capacity(nt);
    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        let mag: f64 = buf.iter().map(|v| v * v).sum::<f64>().sqrt();
        data.push(mag);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(result_name, data, 1)));
    pd
}

fn array_op<F>(input: &PolyData, name_a: &str, name_b: &str, result_name: &str, op: F) -> PolyData
where
    F: Fn(f64, f64) -> f64,
{
    let arr_a = match input.point_data().get_array(name_a) {
        Some(a) => a,
        None => return input.clone(),
    };
    let arr_b = match input.point_data().get_array(name_b) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc = arr_a.num_components().min(arr_b.num_components());
    let nt = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut data = Vec::with_capacity(nt * nc);
    let mut buf_a = vec![0.0f64; nc];
    let mut buf_b = vec![0.0f64; nc];
    for i in 0..nt {
        arr_a.tuple_as_f64(i, &mut buf_a);
        arr_b.tuple_as_f64(i, &mut buf_b);
        for c in 0..nc {
            data.push(op(buf_a[c], buf_b[c]));
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            data,
            nc,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_data() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        pd
    }

    #[test]
    fn add_arrays() {
        let pd = make_test_data();
        let result = array_add(&pd, "a", "b", "sum");
        let arr = result.point_data().get_array("sum").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 11.0).abs() < 1e-10);
    }

    #[test]
    fn scale_array() {
        let pd = make_test_data();
        let result = array_scale(&pd, "a", 5.0, "scaled");
        let arr = result.point_data().get_array("scaled").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(2, &mut val);
        assert!((val[0] - 15.0).abs() < 1e-10);
    }

    #[test]
    fn vector_magnitude() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0, 0.0],
                3,
            )));
        let result = array_magnitude(&pd, "v", "mag");
        let arr = result.point_data().get_array("mag").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10);
    }
}
