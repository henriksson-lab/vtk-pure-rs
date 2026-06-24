use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute point pair features (PPF) for each pair of nearby points.
///
/// PPF = (||d||, angle(n1,d), angle(n2,d), angle(n1,n2)) where d = p2-p1.
/// Adds "PPFCount" (number of similar PPF matches) as a descriptor.
/// Simplified: just counts neighbors within radius.
pub fn point_pair_features(input: &PolyData, radius: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = crate::data::KdTree::build(&pts);

    let normals: Option<Vec<[f64; 3]>> = input.point_data().get_array("Normals").map(|arr| {
        let mut buf = [0.0f64; 3];
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf
            })
            .collect()
    });

    let mut feature_count = vec![0.0f64; n];

    for i in 0..n {
        let nbrs = tree.find_within_radius(pts[i], radius);
        if let Some(ref norms) = normals {
            // Count neighbors with similar normal direction
            let ni = norms[i];
            let mut similar = 0;
            for &(j, _) in &nbrs {
                if j == i {
                    continue;
                }
                let nj = norms[j];
                let dot = (ni[0] * nj[0] + ni[1] * nj[1] + ni[2] * nj[2]).abs();
                if dot > 0.7 {
                    similar += 1;
                } // similar normal
            }
            feature_count[i] = similar as f64;
        } else {
            feature_count[i] = (nbrs.len() as f64) - 1.0; // exclude self
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PPFCount",
            feature_count,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppf_basic() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64 * 0.1, 0.0, 0.0]);
        }

        let result = point_pair_features(&pd, 0.5);
        assert!(result.point_data().get_array("PPFCount").is_some());
    }

    #[test]
    fn with_normals() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));

        let result = point_pair_features(&pd, 1.0);
        let arr = result.point_data().get_array("PPFCount").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] >= 1.0); // point 1 is nearby with similar normal
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = point_pair_features(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
