use crate::data::{AnyDataArray, DataArray, Points, PolyData};
use std::collections::VecDeque;

/// VTK-style Euclidean clustering for point clouds.
///
/// Clusters are connected components under a local radius neighborhood, matching
/// vtkEuclideanClusterExtraction's wave propagation. The `min_points` argument
/// is retained for API compatibility and is not part of VTK's cluster test.
///
/// Returns a PolyData with points and a "ClusterId" point data array.
pub fn euclidean_cluster(input: &PolyData, radius: f64, _min_points: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let radius2 = radius * radius;
    let mut cluster_ids = vec![-1i32; n];
    let mut visited = vec![false; n];
    let mut current_cluster = 0i32;

    // Preload points for faster access
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for i in 0..n {
        if visited[i] {
            continue;
        }
        let mut queue = VecDeque::new();
        visited[i] = true;
        queue.push_back(i);

        while let Some(qi) = queue.pop_front() {
            cluster_ids[qi] = current_cluster;

            for nb in range_query(&pts, qi, radius2) {
                if !visited[nb] {
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
        }

        current_cluster += 1;
    }

    // Build output
    let mut out_points = Points::<f64>::new();
    for i in 0..n {
        out_points.push(pts[i]);
    }

    let cluster_f64: Vec<f64> = cluster_ids.iter().map(|&c| c as f64).collect();

    let mut pd = PolyData::new();
    pd.points = out_points;
    for array in input.point_data().iter().cloned() {
        pd.point_data_mut().add_array(array);
    }
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ClusterId",
            cluster_f64,
            1,
        )));
    pd.point_data_mut().set_active_scalars("ClusterId");
    pd
}

fn range_query(pts: &[[f64; 3]], idx: usize, eps2: f64) -> Vec<usize> {
    let p = pts[idx];
    let mut result = Vec::new();
    for (i, q) in pts.iter().enumerate() {
        let dx = p[0] - q[0];
        let dy = p[1] - q[1];
        let dz = p[2] - q[2];
        if dx * dx + dy * dy + dz * dz <= eps2 {
            result.push(i);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clusters() {
        let mut pd = PolyData::new();
        // Cluster A: 5 points near origin
        for i in 0..5 {
            pd.points.push([i as f64 * 0.1, 0.0, 0.0]);
        }
        // Cluster B: 5 points far away
        for i in 0..5 {
            pd.points.push([100.0 + i as f64 * 0.1, 0.0, 0.0]);
        }

        let result = euclidean_cluster(&pd, 0.5, 3);
        let arr = result.point_data().get_array("ClusterId").unwrap();

        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0] as i32;
        arr.tuple_as_f64(5, &mut buf);
        let c1 = buf[0] as i32;

        assert!(c0 >= 0);
        assert!(c1 >= 0);
        assert_ne!(c0, c1);
    }

    #[test]
    fn isolated_points_are_singleton_clusters() {
        let mut pd = PolyData::new();
        // Isolated points far apart
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);
        pd.points.push([200.0, 0.0, 0.0]);

        let result = euclidean_cluster(&pd, 1.0, 3);
        let arr = result.point_data().get_array("ClusterId").unwrap();
        let mut buf = [0.0f64];

        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], i as f64);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = euclidean_cluster(&pd, 1.0, 2);
        assert_eq!(result.points.len(), 0);
    }
}
