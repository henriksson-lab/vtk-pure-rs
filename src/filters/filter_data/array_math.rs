use crate::data::{AnyDataArray, DataArray, PolyData};

/// Add two point data scalar arrays element-wise. Result stored as `output_name`.
pub fn array_add(input: &PolyData, a_name: &str, b_name: &str, output_name: &str) -> PolyData {
    array_binary_op(input, a_name, b_name, output_name, |a, b| a + b)
}

/// Subtract two point data scalar arrays element-wise (a - b).
pub fn array_subtract(input: &PolyData, a_name: &str, b_name: &str, output_name: &str) -> PolyData {
    array_binary_op(input, a_name, b_name, output_name, |a, b| a - b)
}

/// Multiply two point data scalar arrays element-wise.
pub fn array_multiply(input: &PolyData, a_name: &str, b_name: &str, output_name: &str) -> PolyData {
    array_binary_op(input, a_name, b_name, output_name, |a, b| a * b)
}

/// Divide two point data scalar arrays element-wise (a / b, safe division).
pub fn array_divide(input: &PolyData, a_name: &str, b_name: &str, output_name: &str) -> PolyData {
    array_binary_op(input, a_name, b_name, output_name, |a, b| {
        if b.abs() > 1e-15 {
            a / b
        } else {
            0.0
        }
    })
}

fn array_binary_op<F: Fn(f64, f64) -> f64>(
    input: &PolyData,
    a_name: &str,
    b_name: &str,
    output_name: &str,
    op: F,
) -> PolyData {
    let arr_a = match input.point_data().get_array(a_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let arr_b = match input.point_data().get_array(b_name) {
        Some(b) => b,
        None => return input.clone(),
    };
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ba = [0.0f64];
    let mut bb = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut ba);
            arr_b.tuple_as_f64(i, &mut bb);
            op(ba[0], bb[0])
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output_name,
            values,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pd() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![10.0, 20.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![3.0, 5.0],
                1,
            )));
        pd
    }

    #[test]
    fn add_arrays() {
        let pd = make_pd();
        let r = array_add(&pd, "a", "b", "sum");
        let arr = r.point_data().get_array("sum").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 13.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 25.0);
    }

    #[test]
    fn subtract_arrays() {
        let pd = make_pd();
        let r = array_subtract(&pd, "a", "b", "diff");
        let arr = r.point_data().get_array("diff").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 7.0);
    }

    #[test]
    fn divide_safe() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![10.0], 1)));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![0.0], 1)));
        let r = array_divide(&pd, "a", "b", "ratio");
        let arr = r.point_data().get_array("ratio").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // safe div by zero
    }

    #[test]
    fn missing_array() {
        let pd = make_pd();
        let r = array_add(&pd, "a", "missing", "out");
        assert!(r.point_data().get_array("out").is_none());
    }
}
