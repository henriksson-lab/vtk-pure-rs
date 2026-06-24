use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Sample a scalar field along an existing polyline.
///
/// For each point on the polyline, finds the nearest input point
/// and copies its scalar value. Also computes arc-length parameter.
pub fn sample_along_polyline(input: &PolyData, array_name: &str, polyline: &PolyData) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return polyline.clone(),
    };

    let n_input = input.points.len();
    if n_input == 0 {
        return polyline.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n_input).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let n_line = polyline.points.len();
    let mut values = Vec::with_capacity(n_line);
    let mut arc_len = Vec::with_capacity(n_line);
    let mut buf = [0.0f64];
    let mut cumulative = 0.0;

    for i in 0..n_line {
        let p = polyline.points.get(i);

        if i > 0 {
            let prev = polyline.points.get(i - 1);
            let dx = p[0] - prev[0];
            let dy = p[1] - prev[1];
            let dz = p[2] - prev[2];
            cumulative += (dx * dx + dy * dy + dz * dz).sqrt();
        }
        arc_len.push(cumulative);

        if let Some((nearest, _)) = tree.nearest(p) {
            arr.tuple_as_f64(nearest, &mut buf);
            values.push(buf[0]);
        } else {
            values.push(0.0);
        }
    }

    let mut pd = polyline.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ArcLength",
            arc_len,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_basic() {
        let mut input = PolyData::new();
        input.points.push([0.0, 0.0, 0.0]);
        input.points.push([1.0, 0.0, 0.0]);
        input.points.push([2.0, 0.0, 0.0]);
        input
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0, 30.0],
                1,
            )));

        let mut line = PolyData::new();
        line.points.push([0.0, 0.0, 0.0]);
        line.points.push([1.0, 0.0, 0.0]);
        line.points.push([2.0, 0.0, 0.0]);
        line.lines.push_cell(&[0, 1, 2]);

        let result = sample_along_polyline(&input, "temp", &line);
        assert!(result.point_data().get_array("temp").is_some());
        assert!(result.point_data().get_array("ArcLength").is_some());

        let arr = result.point_data().get_array("ArcLength").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let input = PolyData::new();
        let line = PolyData::new();
        let result = sample_along_polyline(&input, "nope", &line);
        assert_eq!(result.points.len(), 0);
    }
}
