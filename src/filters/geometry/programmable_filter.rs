use crate::data::{AnyDataArray, DataArray, PolyData};

/// Apply a user-defined function to each point of a PolyData.
///
/// The function receives the point position [x, y, z] and returns a scalar value.
/// The results are stored in a new point data array with the given name.
pub fn programmable_filter<F>(input: &PolyData, name: &str, func: F) -> PolyData
where
    F: Fn([f64; 3]) -> f64,
{
    let n = input.points.len();
    let mut values = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        values.push(func(p));
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, values, 1)));
    pd
}

/// Apply a user-defined vector function to each point.
///
/// The function returns a 3-component vector for each point position.
pub fn programmable_filter_vector<F>(input: &PolyData, name: &str, func: F) -> PolyData
where
    F: Fn([f64; 3]) -> [f64; 3],
{
    let n = input.points.len();
    let mut values = Vec::with_capacity(n * 3);

    for i in 0..n {
        let p = input.points.get(i);
        let v = func(p);
        values.push(v[0]);
        values.push(v[1]);
        values.push(v[2]);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, values, 3)));
    pd
}

/// Transform point positions using a user-defined function.
///
/// The function maps each position to a new position.
pub fn programmable_transform<F>(input: &PolyData, func: F) -> PolyData
where
    F: Fn([f64; 3]) -> [f64; 3],
{
    let n = input.points.len();
    let mut pd = input.clone();

    let mut new_points = crate::data::Points::<f64>::new();
    for i in 0..n {
        new_points.push(func(input.points.get(i)));
    }
    pd.points = new_points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tri() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.points.push([7.0, 8.0, 9.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd
    }

    #[test]
    fn scalar_function() {
        let pd = make_tri();
        let result = programmable_filter(&pd, "dist", |p| {
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
        });
        let arr = result.point_data().get_array("dist").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - (1.0 + 4.0 + 9.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn vector_function() {
        let pd = make_tri();
        let result =
            programmable_filter_vector(&pd, "doubled", |p| [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0]);
        let arr = result.point_data().get_array("doubled").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [2.0, 4.0, 6.0]);
    }

    #[test]
    fn transform_positions() {
        let pd = make_tri();
        let result = programmable_transform(&pd, |p| [p[0] + 10.0, p[1], p[2]]);
        let p = result.points.get(0);
        assert_eq!(p[0], 11.0);
    }

    #[test]
    fn preserves_topology() {
        let pd = make_tri();
        let result = programmable_filter(&pd, "x", |p| p[0]);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
