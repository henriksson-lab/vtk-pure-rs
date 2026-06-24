//! Simple mesh reconstruction from point cloud using k-nearest neighbor triangulation.
use crate::data::{CellArray, Points, PolyData};

pub fn mesh_from_point_cloud(cloud: &PolyData, k: usize) -> PolyData {
    let n = cloud.points.len();
    if n < 3 {
        return cloud.clone();
    }
    let k = k.max(3).min(n - 1);
    // Find k nearest neighbors for each point
    let mut polys = CellArray::new();
    let mut created = std::collections::HashSet::new();
    for i in 0..n {
        let pi = cloud.points.get(i);
        let mut dists: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                let pj = cloud.points.get(j);
                (
                    j,
                    (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2),
                )
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let neighbors: Vec<usize> = dists.iter().take(k).map(|&(j, _)| j).collect();
        // Create triangles with pairs of nearest neighbors
        for ni in 0..neighbors.len() {
            for nj in (ni + 1)..neighbors.len() {
                let mut tri = [i, neighbors[ni], neighbors[nj]];
                tri.sort();
                if created.insert((tri[0], tri[1], tri[2])) {
                    polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
                }
                if created.len() > n * 3 {
                    break;
                } // limit triangle count
            }
            if created.len() > n * 3 {
                break;
            }
        }
    }
    let mut result = PolyData::new();
    result.points = cloud.points.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_cloud() {
        let mut cloud = PolyData::new();
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([1.0, 0.0, 0.0]);
        pts.push([0.5, 1.0, 0.0]);
        pts.push([1.5, 0.5, 0.0]);
        cloud.points = pts;
        let r = mesh_from_point_cloud(&cloud, 3);
        assert!(r.polys.num_cells() > 0);
    }
}
