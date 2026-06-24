use crate::data::{AnyDataArray, DataArray, PolyData};

/// Apply a user-defined function to compute a new scalar array from
/// existing point data.
///
/// The function receives the point index, coordinates, and access to
/// existing arrays, and returns a scalar value for each point.
pub fn calculator<F>(input: &PolyData, result_name: &str, f: F) -> PolyData
where
    F: Fn(usize, [f64; 3], &crate::data::DataSetAttributes) -> f64,
{
    let n = input.points.len();
    let mut values = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        values.push(f(i, p, input.point_data()));
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            values,
            1,
        )));
    pd
}

/// Apply a vector-valued function to compute a new 3-component array.
pub fn calculator_vector<F>(input: &PolyData, result_name: &str, f: F) -> PolyData
where
    F: Fn(usize, [f64; 3], &crate::data::DataSetAttributes) -> [f64; 3],
{
    let n = input.points.len();
    let mut values = Vec::with_capacity(n * 3);

    for i in 0..n {
        let p = input.points.get(i);
        let v = f(i, p, input.point_data());
        values.extend_from_slice(&v);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            values,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_radius() {
        let mut pd = PolyData::new();
        pd.points.push([3.0, 4.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = calculator(&pd, "radius", |_i, p, _attrs| {
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
        });

        let arr = result.point_data().get_array("radius").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn compute_from_existing_scalar() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let temp = DataArray::from_vec("temp", vec![100.0, 200.0], 1);
        pd.point_data_mut().add_array(temp.into());
        pd.point_data_mut().set_active_scalars("temp");

        let result = calculator(&pd, "temp_f", |i, _p, attrs| {
            let mut buf = [0.0f64];
            if let Some(s) = attrs.scalars() {
                s.tuple_as_f64(i, &mut buf);
            }
            buf[0] * 1.8 + 32.0 // Celsius to Fahrenheit
        });

        let arr = result.point_data().get_array("temp_f").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 212.0).abs() < 1e-10); // 100°C = 212°F
    }

    #[test]
    fn compute_vector() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);

        let result = calculator_vector(&pd, "doubled", |_i, p, _attrs| {
            [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0]
        });

        let arr = result.point_data().get_array("doubled").unwrap();
        assert_eq!(arr.num_components(), 3);
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);
    }
}
