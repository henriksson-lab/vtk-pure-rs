use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Find nearest-point correspondences between two point sets.
///
/// For each point in `source`, finds the nearest point in `target`.
/// Adds "CorrespondenceIndex" (target point ID) and "CorrespondenceDistance"
/// arrays to the source.
pub fn find_correspondences(source: &PolyData, target: &PolyData) -> PolyData {
    let n_src = source.points.len();
    let n_tgt = target.points.len();
    if n_src == 0 || n_tgt == 0 {
        return source.clone();
    }

    let tgt_pts: Vec<[f64; 3]> = (0..n_tgt).map(|i| target.points.get(i)).collect();
    let tree = KdTree::build(&tgt_pts);

    let mut indices = Vec::with_capacity(n_src);
    let mut distances = Vec::with_capacity(n_src);

    for i in 0..n_src {
        if let Some((idx, d2)) = tree.nearest(source.points.get(i)) {
            indices.push(idx as f64);
            distances.push(d2.sqrt());
        } else {
            indices.push(-1.0);
            distances.push(f64::MAX);
        }
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CorrespondenceIndex",
            indices,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CorrespondenceDistance",
            distances,
            1,
        )));
    pd
}

/// Transfer a scalar array from target to source using nearest-point correspondences.
pub fn transfer_attribute(source: &PolyData, target: &PolyData, array_name: &str) -> PolyData {
    let arr = match target.point_data().get_array(array_name) {
        Some(a) => a,
        None => return source.clone(),
    };

    let n_src = source.points.len();
    let n_tgt = target.points.len();
    if n_src == 0 || n_tgt == 0 {
        return source.clone();
    }

    let tgt_pts: Vec<[f64; 3]> = (0..n_tgt).map(|i| target.points.get(i)).collect();
    let tree = KdTree::build(&tgt_pts);

    let nc = arr.num_components();
    let mut values = Vec::with_capacity(n_src * nc);
    let mut buf = vec![0.0f64; nc];

    for i in 0..n_src {
        if let Some((idx, _)) = tree.nearest(source.points.get(i)) {
            arr.tuple_as_f64(idx, &mut buf);
            values.extend_from_slice(&buf);
        } else {
            for _ in 0..nc {
                values.push(0.0);
            }
        }
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, nc,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correspondences() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        src.points.push([5.0, 0.0, 0.0]);

        let mut tgt = PolyData::new();
        tgt.points.push([0.1, 0.0, 0.0]);
        tgt.points.push([4.9, 0.0, 0.0]);

        let result = find_correspondences(&src, &tgt);
        let idx = result
            .point_data()
            .get_array("CorrespondenceIndex")
            .unwrap();
        let mut buf = [0.0f64];
        idx.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // nearest to tgt[0]
        idx.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0); // nearest to tgt[1]
    }

    #[test]
    fn transfer() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);

        let mut tgt = PolyData::new();
        tgt.points.push([0.1, 0.0, 0.0]);
        tgt.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![42.0],
                1,
            )));

        let result = transfer_attribute(&src, &tgt, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 42.0);
    }

    #[test]
    fn missing_attribute() {
        let src = PolyData::new();
        let tgt = PolyData::new();
        let result = transfer_attribute(&src, &tgt, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
