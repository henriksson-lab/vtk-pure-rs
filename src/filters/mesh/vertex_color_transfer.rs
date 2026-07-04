use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Transfer vertex colors from one mesh to another via nearest-point matching.
///
/// For each vertex in `target`, finds the nearest vertex in `source`
/// and copies the named array value. Works for any scalar or vector array.
pub fn vertex_color_transfer(source: &PolyData, target: &PolyData, array_name: &str) -> PolyData {
    let arr = match source.point_data().get_array(array_name) {
        Some(a) => a,
        None => return target.clone(),
    };

    let ns = source.points.len();
    let nt = target.points.len();
    if ns == 0 || nt == 0 {
        return target.clone();
    }
    if arr.num_tuples() != ns {
        return target.clone();
    }

    let src_pts: Vec<[f64; 3]> = (0..ns).map(|i| source.points.get(i)).collect();
    let tree = KdTree::build(&src_pts);

    let nc = arr.num_components();
    let mut values = Vec::with_capacity(nt * nc);
    let mut buf = vec![0.0f64; nc];

    for i in 0..nt {
        if let Some((idx, _)) = tree.nearest(target.points.get(i)) {
            arr.tuple_as_f64(idx, &mut buf);
            values.extend_from_slice(&buf);
        } else {
            for _ in 0..nc {
                values.push(0.0);
            }
        }
    }

    let mut pd = target.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, nc,
        )));
    pd
}

/// Transfer all point data arrays from source to target.
pub fn transfer_all_point_data(source: &PolyData, target: &PolyData) -> PolyData {
    let mut pd = target.clone();
    for i in 0..source.point_data().num_arrays() {
        let arr = source.point_data().get_array_by_index(i).unwrap();
        pd = vertex_color_transfer(source, &pd, arr.name());
    }
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_colors() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        src.points.push([1.0, 0.0, 0.0]);
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "color",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));

        let mut tgt = PolyData::new();
        tgt.points.push([0.1, 0.0, 0.0]); // nearest to src[0]

        let result = vertex_color_transfer(&src, &tgt, "color");
        let arr = result.point_data().get_array("color").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn transfer_all() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![2.0], 1)));

        let mut tgt = PolyData::new();
        tgt.points.push([0.0, 0.0, 0.0]);

        let result = transfer_all_point_data(&src, &tgt);
        assert!(result.point_data().get_array("a").is_some());
        assert!(result.point_data().get_array("b").is_some());
    }

    #[test]
    fn missing_array() {
        let src = PolyData::new();
        let mut tgt = PolyData::new();
        tgt.points.push([0.0, 0.0, 0.0]);
        let result = vertex_color_transfer(&src, &tgt, "nope");
        assert!(result.point_data().get_array("nope").is_none());
    }
}
