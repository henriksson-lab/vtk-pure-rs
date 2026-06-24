use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Estimate the local feature size at each vertex.
///
/// For each point, the local feature size is the distance to the
/// nearest point in the medial axis, approximated by the distance
/// to the second-nearest point. Adds "FeatureSize" scalar.
pub fn local_feature_size(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 2 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut sizes = Vec::with_capacity(n);
    for i in 0..n {
        let knn = tree.k_nearest(pts[i], 2);
        // Second nearest (first is self with d=0)
        let d = if knn.len() >= 2 { knn[1].1.sqrt() } else { 0.0 };
        sizes.push(d);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FeatureSize",
            sizes,
            1,
        )));
    pd
}

/// Compute the average inter-point spacing of a point set.
pub fn average_spacing(input: &PolyData, k: usize) -> f64 {
    let n = input.points.len();
    if n < 2 {
        return 0.0;
    }
    let k = k.max(1).min(n - 1);

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut total = 0.0;
    for i in 0..n {
        let knn = tree.k_nearest(pts[i], k + 1);
        let sum_d: f64 = knn.iter().skip(1).map(|&(_, d2)| d2.sqrt()).sum();
        total += sum_d / k as f64;
    }
    total / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_size_uniform() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }

        let result = local_feature_size(&pd);
        let arr = result.point_data().get_array("FeatureSize").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10); // uniform spacing = 1
    }

    #[test]
    fn avg_spacing() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64 * 2.0, 0.0, 0.0]);
        }

        let s = average_spacing(&pd, 1);
        assert!((s - 2.0).abs() < 1e-10); // spacing = 2
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(average_spacing(&pd, 1), 0.0);
    }

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = local_feature_size(&pd);
        assert_eq!(result.points.len(), 1);
    }
}
