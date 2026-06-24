use crate::data::{AnyDataArray, CellArray, DataArray, KdTree, Points, PolyData};

/// Sample scalar values from a PolyData along a line between two points.
///
/// Creates `num_samples` evenly-spaced points along the line from `p0` to `p1`,
/// and for each, finds the nearest point in the input and copies its scalar value.
/// Returns a PolyData with the sample points, a line cell, and the sampled scalar.
pub fn sample_along_line(
    input: &PolyData,
    array_name: &str,
    p0: [f64; 3],
    p1: [f64; 3],
    num_samples: usize,
) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    if n == 0 || num_samples < 2 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut out_points = Points::<f64>::new();
    let mut values = Vec::with_capacity(num_samples);
    let mut arc_length = Vec::with_capacity(num_samples);
    let mut line_ids = Vec::with_capacity(num_samples);
    let mut buf = [0.0f64];

    let total_len =
        ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2) + (p1[2] - p0[2]).powi(2)).sqrt();

    for i in 0..num_samples {
        let t = i as f64 / (num_samples - 1) as f64;
        let p = [
            p0[0] + t * (p1[0] - p0[0]),
            p0[1] + t * (p1[1] - p0[1]),
            p0[2] + t * (p1[2] - p0[2]),
        ];

        let idx = out_points.len() as i64;
        out_points.push(p);
        line_ids.push(idx);
        arc_length.push(t * total_len);

        if let Some((nearest, _)) = tree.nearest(p) {
            arr.tuple_as_f64(nearest, &mut buf);
            values.push(buf[0]);
        } else {
            values.push(0.0);
        }
    }

    let mut lines = CellArray::new();
    lines.push_cell(&line_ids);

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = lines;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ArcLength",
            arc_length,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_along() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0, 30.0],
                1,
            )));

        let result = sample_along_line(&pd, "temp", [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 5);
        assert_eq!(result.points.len(), 5);
        assert_eq!(result.lines.num_cells(), 1);
        assert!(result.point_data().get_array("temp").is_some());
        assert!(result.point_data().get_array("ArcLength").is_some());
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = sample_along_line(&pd, "nope", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 10);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn endpoint_values() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![100.0, 200.0],
                1,
            )));

        let result = sample_along_line(&pd, "v", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0], 3);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 100.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 200.0);
    }
}
