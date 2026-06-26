use crate::data::{AnyDataArray, DataArray, PolyData};

/// Add two named point-data arrays element-wise.
pub fn array_add(input: &PolyData, a: &str, b: &str, result_name: &str) -> PolyData {
    binary_op(input, a, b, result_name, |x, y| x + y)
}

/// Subtract array `b` from array `a` element-wise.
pub fn array_subtract(input: &PolyData, a: &str, b: &str, result_name: &str) -> PolyData {
    binary_op(input, a, b, result_name, |x, y| x - y)
}

/// Multiply two named point-data arrays element-wise.
pub fn array_multiply(input: &PolyData, a: &str, b: &str, result_name: &str) -> PolyData {
    binary_op(input, a, b, result_name, |x, y| x * y)
}

/// Divide array `a` by array `b` element-wise (0/0 -> 0).
pub fn array_divide(input: &PolyData, a: &str, b: &str, result_name: &str) -> PolyData {
    binary_op(input, a, b, result_name, |x, y| {
        if y.abs() < 1e-300 {
            0.0
        } else {
            x / y
        }
    })
}

/// Scale an array by a constant factor.
pub fn array_scale(input: &PolyData, a: &str, factor: f64, result_name: &str) -> PolyData {
    unary_op(input, a, result_name, |x| x * factor)
}

/// Compute the element-wise square root of an array (negative values -> 0).
pub fn array_sqrt(input: &PolyData, a: &str, result_name: &str) -> PolyData {
    unary_op(
        input,
        a,
        result_name,
        |x| {
            if x < 0.0 {
                0.0
            } else {
                x.sqrt()
            }
        },
    )
}

/// Compute the element-wise absolute value of an array.
pub fn array_abs(input: &PolyData, a: &str, result_name: &str) -> PolyData {
    unary_op(input, a, result_name, |x| x.abs())
}

fn binary_op<F>(input: &PolyData, name_a: &str, name_b: &str, result_name: &str, op: F) -> PolyData
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

    let nc_a = arr_a.num_components();
    let nc_b = arr_b.num_components();
    let nc = match (nc_a, nc_b) {
        (a, b) if a == b => a,
        (1, b) => b,
        (a, 1) => a,
        _ => return input.clone(),
    };
    let nt = arr_a.num_tuples();
    if nt != arr_b.num_tuples() {
        return input.clone();
    }

    let mut data: Vec<f64> = Vec::with_capacity(nt * nc);
    let mut buf_a = vec![0.0f64; nc_a];
    let mut buf_b = vec![0.0f64; nc_b];
    for i in 0..nt {
        arr_a.tuple_as_f64(i, &mut buf_a);
        arr_b.tuple_as_f64(i, &mut buf_b);
        for c in 0..nc {
            let x = buf_a[if nc_a == 1 { 0 } else { c }];
            let y = buf_b[if nc_b == 1 { 0 } else { c }];
            data.push(op(x, y));
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

fn unary_op<F>(input: &PolyData, name: &str, result_name: &str, op: F) -> PolyData
where
    F: Fn(f64) -> f64,
{
    let arr = match input.point_data().get_array(name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc = arr.num_components();
    let nt = arr.num_tuples();
    let mut data: Vec<f64> = Vec::with_capacity(nt * nc);
    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        for c in 0..nc {
            data.push(op(buf[c]));
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
                vec![4.0, 9.0, 16.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![2.0, 3.0, 4.0],
                1,
            )));
        pd
    }

    #[test]
    fn add_and_subtract() {
        let pd = make_test_data();
        let sum = array_add(&pd, "a", "b", "sum");
        let arr = sum.point_data().get_array("sum").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 6.0).abs() < 1e-10);

        let diff = array_subtract(&pd, "a", "b", "diff");
        let arr = diff.point_data().get_array("diff").unwrap();
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn scale_and_sqrt() {
        let pd = make_test_data();
        let scaled = array_scale(&pd, "a", 0.5, "half");
        let arr = scaled.point_data().get_array("half").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);

        let rooted = array_sqrt(&pd, "a", "root");
        let arr = rooted.point_data().get_array("root").unwrap();
        arr.tuple_as_f64(2, &mut val);
        assert!((val[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn divide_and_abs() {
        let pd = make_test_data();
        let divided = array_divide(&pd, "a", "b", "div");
        let arr = divided.point_data().get_array("div").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 3.0).abs() < 1e-10);

        // Test abs with negative values
        let mut pd2 = PolyData::new();
        pd2.points.push([0.0, 0.0, 0.0]);
        pd2.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("neg", vec![-5.0], 1)));
        let result = array_abs(&pd2, "neg", "pos");
        let arr = result.point_data().get_array("pos").unwrap();
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10);
    }
}
