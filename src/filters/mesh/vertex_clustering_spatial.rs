//! Spatial vertex clustering for mesh simplification and point cloud reduction.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Cluster vertices into a regular grid and merge each cluster into one vertex.
///
/// This is a fast O(n) simplification method.
pub fn cluster_vertices_grid(mesh: &PolyData, grid_size: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    // Assign each vertex to a grid cell
    let mut cell_map: std::collections::HashMap<[i64; 3], Vec<usize>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let key = [
            (p[0] / grid_size).floor() as i64,
            (p[1] / grid_size).floor() as i64,
            (p[2] / grid_size).floor() as i64,
        ];
        cell_map.entry(key).or_default().push(i);
    }

    // Compute cluster centroids
    let mut new_points = Points::<f64>::new();
    let mut vertex_remap = vec![0usize; n];
    for (_, indices) in &cell_map {
        let new_idx = new_points.len();
        let mut avg = [0.0; 3];
        for &i in indices {
            let p = mesh.points.get(i);
            for c in 0..3 {
                avg[c] += p[c];
            }
        }
        let k = indices.len() as f64;
        new_points.push([avg[0] / k, avg[1] / k, avg[2] / k]);
        for &i in indices {
            vertex_remap[i] = new_idx;
        }
    }

    // Remap triangles, skip degenerate
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&pid| vertex_remap[pid as usize] as i64)
            .collect();
        let unique: std::collections::HashSet<i64> = remapped.iter().cloned().collect();
        if unique.len() >= 3 {
            new_polys.push_cell(&remapped);
        }
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

/// Cluster a point cloud into N clusters using farthest point sampling.
pub fn farthest_point_cluster(mesh: &PolyData, n_clusters: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || n_clusters == 0 {
        return mesh.clone();
    }
    let nc = n_clusters.min(n);

    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut centers = vec![0usize; nc];
    let mut min_dist = vec![f64::MAX; n];
    centers[0] = 0;

    for ci in 1..nc {
        // Update min distances
        let prev = centers[ci - 1];
        for i in 0..n {
            let d = dist2(pts[i], pts[prev]);
            min_dist[i] = min_dist[i].min(d);
        }
        // Pick farthest point
        let farthest = (0..n)
            .max_by(|&a, &b| {
                min_dist[a]
                    .partial_cmp(&min_dist[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        centers[ci] = farthest;
    }

    // Assign each point to nearest center
    let mut labels = vec![0usize; n];
    for i in 0..n {
        let mut best = 0;
        let mut best_d = f64::MAX;
        for (ci, &c) in centers.iter().enumerate() {
            let d = dist2(pts[i], pts[c]);
            if d < best_d {
                best_d = d;
                best = ci;
            }
        }
        labels[i] = best;
    }

    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("ClusterId", data, 1)));
    result
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_cluster() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64 * 0.1, y as f64 * 0.1, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let simplified = cluster_vertices_grid(&mesh, 0.3);
        assert!(simplified.points.len() < mesh.points.len());
        assert!(simplified.points.len() > 0);
    }
    #[test]
    fn farthest_point() {
        let mesh = PolyData::from_points((0..50).map(|i| [i as f64, 0.0, 0.0]).collect::<Vec<_>>());
        let result = farthest_point_cluster(&mesh, 5);
        assert!(result.point_data().get_array("ClusterId").is_some());
    }
}
