use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Cluster mesh vertices into N groups using farthest-point sampling.
///
/// Selects N seed points by iteratively picking the point farthest
/// from all existing seeds, then assigns each vertex to its nearest seed.
/// Adds a "ClusterId" scalar array.
pub fn vertex_cluster(input: &PolyData, n_clusters: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let k = n_clusters.max(1).min(n);

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Farthest point sampling for seeds
    let mut seeds = vec![0usize]; // start with first point
    let mut min_dist = vec![f64::MAX; n];

    for _ in 1..k {
        let last_seed = *seeds.last().unwrap();
        // Update min distances
        for i in 0..n {
            let d2 = dist2(pts[i], pts[last_seed]);
            min_dist[i] = min_dist[i].min(d2);
        }
        // Find farthest point
        let farthest = (0..n)
            .max_by(|&a, &b| min_dist[a].partial_cmp(&min_dist[b]).unwrap())
            .unwrap();
        seeds.push(farthest);
    }

    // Build k-d tree of seeds for nearest-neighbor assignment
    let seed_pts: Vec<[f64; 3]> = seeds.iter().map(|&s| pts[s]).collect();
    let tree = KdTree::build(&seed_pts);

    let mut cluster_ids = vec![0.0f64; n];
    for i in 0..n {
        if let Some((idx, _)) = tree.nearest(pts[i]) {
            cluster_ids[i] = idx as f64;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ClusterId",
            cluster_ids,
            1,
        )));
    pd
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clusters() {
        let mut pd = PolyData::new();
        // Two groups of points
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        for i in 0..5 {
            pd.points.push([100.0 + i as f64, 0.0, 0.0]);
        }

        let result = vertex_cluster(&pd, 2);
        let arr = result.point_data().get_array("ClusterId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let id0 = buf[0];
        arr.tuple_as_f64(5, &mut buf);
        let id1 = buf[0];
        assert_ne!(id0, id1); // different clusters
    }

    #[test]
    fn single_cluster() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = vertex_cluster(&pd, 1);
        let arr = result.point_data().get_array("ClusterId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn more_clusters_than_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = vertex_cluster(&pd, 10);
        assert!(result.point_data().get_array("ClusterId").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = vertex_cluster(&pd, 5);
        assert_eq!(result.points.len(), 0);
    }
}
