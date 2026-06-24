use crate::data::{AnyDataArray, DataArray, PolyData};

/// Add bounding box coordinates as point data scalars.
///
/// Adds arrays "X", "Y", "Z" containing each point's coordinates.
/// Useful for coloring or filtering by position.
pub fn add_coordinate_arrays(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut zs = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        xs.push(p[0]);
        ys.push(p[1]);
        zs.push(p[2]);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("X", xs, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Y", ys, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Z", zs, 1)));
    pd
}

/// Compute distance from each point to a reference point.
///
/// Adds a "DistanceTo" scalar array.
pub fn distance_to_point(input: &PolyData, reference: [f64; 3]) -> PolyData {
    let n = input.points.len();
    let mut dists = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        let dx = p[0] - reference[0];
        let dy = p[1] - reference[1];
        let dz = p[2] - reference[2];
        dists.push((dx * dx + dy * dy + dz * dz).sqrt());
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DistanceTo",
            dists,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);

        let result = add_coordinate_arrays(&pd);
        let x = result.point_data().get_array("X").unwrap();
        let y = result.point_data().get_array("Y").unwrap();
        let z = result.point_data().get_array("Z").unwrap();
        let mut buf = [0.0f64];
        x.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        y.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0);
        z.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 6.0);
    }

    #[test]
    fn distance_to() {
        let mut pd = PolyData::new();
        pd.points.push([3.0, 4.0, 0.0]);

        let result = distance_to_point(&pd, [0.0, 0.0, 0.0]);
        let arr = result.point_data().get_array("DistanceTo").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = add_coordinate_arrays(&pd);
        assert!(result.point_data().get_array("X").is_some());
    }
}
