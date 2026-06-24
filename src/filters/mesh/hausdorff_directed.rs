use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Compute directed Hausdorff distance with per-point error coloring.
///
/// For each point in `source`, computes distance to nearest point in `target`.
/// Adds "HausdorffError" scalar and returns (max, mean, rms) distances.
pub fn directed_hausdorff_colored(
    source: &PolyData,
    target: &PolyData,
) -> (PolyData, f64, f64, f64) {
    let ns = source.points.len();
    let nt = target.points.len();
    if ns == 0 || nt == 0 {
        return (source.clone(), 0.0, 0.0, 0.0);
    }

    let tgt_pts: Vec<[f64; 3]> = (0..nt).map(|i| target.points.get(i)).collect();
    let tree = KdTree::build(&tgt_pts);

    let mut errors = Vec::with_capacity(ns);
    let mut max_d = 0.0f64;
    let mut sum_d = 0.0;
    let mut sum_d2 = 0.0;

    for i in 0..ns {
        let d = match tree.nearest(source.points.get(i)) {
            Some((_, d2)) => d2.sqrt(),
            None => 0.0,
        };
        errors.push(d);
        max_d = max_d.max(d);
        sum_d += d;
        sum_d2 += d * d;
    }

    let mean = sum_d / ns as f64;
    let rms = (sum_d2 / ns as f64).sqrt();

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HausdorffError",
            errors,
            1,
        )));
    (pd, max_d, mean, rms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_meshes() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let (result, max_d, mean, _) = directed_hausdorff_colored(&pd, &pd);
        assert_eq!(max_d, 0.0);
        assert_eq!(mean, 0.0);
        assert!(result.point_data().get_array("HausdorffError").is_some());
    }

    #[test]
    fn known_distance() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        let mut tgt = PolyData::new();
        tgt.points.push([3.0, 4.0, 0.0]);
        let (_, max_d, _, _) = directed_hausdorff_colored(&src, &tgt);
        assert!((max_d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let src = PolyData::new();
        let tgt = PolyData::new();
        let (_, max_d, _, _) = directed_hausdorff_colored(&src, &tgt);
        assert_eq!(max_d, 0.0);
    }
}
