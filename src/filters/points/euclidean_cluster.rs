use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::VecDeque;

/// DBSCAN-like Euclidean clustering for point clouds.
///
/// For each unvisited point, finds all points within `epsilon` distance.
/// If the neighbor count is >= `min_points`, a cluster is formed and grown
/// by expanding each neighbor's neighborhood.
///
/// Returns a PolyData with vertex cells and a "ClusterId" point data array.
/// Points not assigned to any cluster get ClusterId = -1.
pub fn euclidean_cluster(input: &PolyData, epsilon: f64, min_points: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let eps2 = epsilon * epsilon;
    let mut cluster_ids = vec![-1i32; n];
    let mut visited = vec![false; n];
    let mut current_cluster = 0i32;

    // Preload points for faster access
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for i in 0..n {
        if visited[i] {
            continue;
        }
        visited[i] = true;

        let neighbors = range_query(&pts, i, eps2);
        if neighbors.len() < min_points {
            continue; // Noise point
        }

        // Start a new cluster
        cluster_ids[i] = current_cluster;
        let mut queue = VecDeque::new();
        for &nb in &neighbors {
            queue.push_back(nb);
        }

        while let Some(qi) = queue.pop_front() {
            if !visited[qi] {
                visited[qi] = true;
                let nb2 = range_query(&pts, qi, eps2);
                if nb2.len() >= min_points {
                    for &nb in &nb2 {
                        queue.push_back(nb);
                    }
                }
            }
            if cluster_ids[qi] < 0 {
                cluster_ids[qi] = current_cluster;
            }
        }

        current_cluster += 1;
    }

    // Build output
    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    for i in 0..n {
        out_points.push(pts[i]);
        out_verts.push_cell(&[i as i64]);
    }

    let cluster_f64: Vec<f64> = cluster_ids.iter().map(|&c| c as f64).collect();

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ClusterId",
            cluster_f64,
            1,
        )));
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
    fn noise_points() {
        let mut pd = PolyData::new();
        // Isolated points far apart
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);
        pd.points.push([200.0, 0.0, 0.0]);

        let result = euclidean_cluster(&pd, 1.0, 3);
        let arr = result.point_data().get_array("ClusterId").unwrap();
        let mut buf = [0.0f64];

        // All should be noise (-1)
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], -1.0);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = euclidean_cluster(&pd, 1.0, 2);
        assert_eq!(result.points.len(), 0);
    }
}
