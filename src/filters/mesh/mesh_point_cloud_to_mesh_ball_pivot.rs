//! Simple ball-pivot-like surface reconstruction from point cloud.
use crate::data::{CellArray, Points, PolyData};
/// Simple triangle fan reconstruction from nearest neighbors.
pub fn reconstruct_surface_knn(points: &PolyData, k: usize) -> PolyData {
    let n = points.points.len();
    if n < 3 {
        return points.clone();
    }
    let k = k.max(3).min(n - 1);
    let pts_v: Vec<[f64; 3]> = (0..n).map(|i| points.points.get(i)).collect();
    let mut polys = CellArray::new();
    let mut seen: std::collections::HashSet<(usize, usize, usize)> =
        std::collections::HashSet::new();
    for i in 0..n {
        let p = pts_v[i];
        let mut dists: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                (
                    j,
                    (pts_v[j][0] - p[0]).powi(2)
                        + (pts_v[j][1] - p[1]).powi(2)
                        + (pts_v[j][2] - p[2]).powi(2),
                )
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let nbs: Vec<usize> = dists.iter().take(k).map(|&(j, _)| j).collect();
        for ni in 0..nbs.len() - 1 {
            let a = i;
            let b = nbs[ni];
            let c = nbs[ni + 1];
            let mut tri = [a, b, c];
            tri.sort();
            if seen.insert((tri[0], tri[1], tri[2])) {
                polys.push_cell(&[a as i64, b as i64, c as i64]);
            }
        }
    }
    let mut r = points.clone();
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut pc = PolyData::new();
        for i in 0..5 {
            for j in 0..5 {
                pc.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let r = reconstruct_surface_knn(&pc, 4);
        assert!(r.polys.num_cells() > 10);
    }
}
