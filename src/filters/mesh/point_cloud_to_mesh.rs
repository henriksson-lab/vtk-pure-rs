use crate::data::{CellArray, KdTree, Points, PolyData};

/// Reconstruct a triangle mesh from a point cloud using ball pivoting.
///
/// Simplified version: connects each point to its k nearest neighbors
/// and creates triangles from triplets of mutually-connected points.
/// Works well for relatively uniform point clouds.
pub fn point_cloud_to_mesh(input: &PolyData, k: usize) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);
    let k = k.max(3).min(n);

    // For each point, find k nearest neighbors
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        let knn = tree.k_nearest(pts[i], k + 1);
        for &(j, _) in &knn {
            if j != i && !adj[i].contains(&j) {
                adj[i].push(j);
                if !adj[j].contains(&i) {
                    adj[j].push(i);
                }
            }
        }
    }

    // Find triangles: triplets where all three are mutually connected
    let mut tris = std::collections::HashSet::new();
    for i in 0..n {
        for &j in &adj[i] {
            if j <= i {
                continue;
            }
            for &m in &adj[i] {
                if m <= j {
                    continue;
                }
                if adj[j].contains(&m) {
                    let mut tri = [i, j, m];
                    tri.sort();
                    tris.insert(tri);
                }
            }
        }
    }

    let mut out_points = Points::<f64>::new();
    for p in &pts {
        out_points.push(*p);
    }

    let mut out_polys = CellArray::new();
    for tri in &tris {
        out_polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_reconstruction() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let result = point_cloud_to_mesh(&pd, 4);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = point_cloud_to_mesh(&pd, 3);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn three_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        let result = point_cloud_to_mesh(&pd, 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = point_cloud_to_mesh(&pd, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
