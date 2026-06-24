use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Compute the distance from each point of `source` to the nearest point of `target`.
///
/// Adds a "Distance" scalar to `source`'s point data.
/// Uses a k-d tree for efficient nearest-neighbor queries.
pub fn poly_data_distance(source: &PolyData, target: &PolyData) -> PolyData {
    let n_src = source.points.len();
    let n_tgt = target.points.len();

    if n_tgt == 0 {
        return source.clone();
    }

    let tgt_pts: Vec<[f64; 3]> = (0..n_tgt).map(|i| target.points.get(i)).collect();
    let tree = KdTree::build(&tgt_pts);

    let mut distances = vec![0.0f64; n_src];
    for i in 0..n_src {
        let p = source.points.get(i);
        if let Some((_idx, d2)) = tree.nearest(p) {
            distances[i] = d2.sqrt();
        }
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    pd
}

/// Compute the symmetric Hausdorff-like distance statistics between two point sets.
///
/// Returns (max_dist_a_to_b, max_dist_b_to_a, mean_a_to_b, mean_b_to_a).
pub fn distance_stats(a: &PolyData, b: &PolyData) -> (f64, f64, f64, f64) {
    let compute = |src: &PolyData, tgt: &PolyData| -> (f64, f64) {
        let n_src = src.points.len();
        let n_tgt = tgt.points.len();
        if n_src == 0 || n_tgt == 0 {
            return (0.0, 0.0);
        }
        let tgt_pts: Vec<[f64; 3]> = (0..n_tgt).map(|i| tgt.points.get(i)).collect();
        let tree = KdTree::build(&tgt_pts);

        let mut max_d = 0.0f64;
        let mut sum_d = 0.0f64;
        for i in 0..n_src {
            if let Some((_, d2)) = tree.nearest(src.points.get(i)) {
                let d = d2.sqrt();
                max_d = max_d.max(d);
                sum_d += d;
            }
        }
        (max_d, sum_d / n_src as f64)
    };

    let (max_ab, mean_ab) = compute(a, b);
    let (max_ba, mean_ba) = compute(b, a);
    (max_ab, max_ba, mean_ab, mean_ba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_to_self() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = poly_data_distance(&pd, &pd);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..2 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn known_distance() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);

        let mut tgt = PolyData::new();
        tgt.points.push([3.0, 4.0, 0.0]);

        let result = poly_data_distance(&src, &tgt);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn symmetric_stats() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([0.0, 1.0, 0.0]);
        b.points.push([1.0, 1.0, 0.0]);

        let (_, _, mean_ab, mean_ba) = distance_stats(&a, &b);
        assert!((mean_ab - 1.0).abs() < 1e-10);
        assert!((mean_ba - 1.0).abs() < 1e-10);
    }
}
