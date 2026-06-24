use crate::data::{CellArray, KdTree, Points, PolyData};
use std::collections::HashMap;

/// Coarsen a mesh by merging clusters of nearby vertices.
///
/// Iteratively selects the vertex with most neighbors and merges
/// all its one-ring neighbors into it. Faster than quadric decimation
/// for aggressive reduction. `target_points` is the desired point count.
pub fn coarsen(input: &PolyData, target_points: usize) -> PolyData {
    let n = input.points.len();
    if n <= target_points {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    // Compute target radius from desired reduction
    let ratio = (n as f64 / target_points.max(1) as f64).sqrt();
    // Find average nearest-neighbor distance
    let mut avg_d = 0.0;
    let sample_n = n.min(100);
    for i in 0..sample_n {
        let knn = tree.k_nearest(pts[i], 2);
        if knn.len() >= 2 {
            avg_d += knn[1].1.sqrt();
        }
    }
    avg_d /= sample_n as f64;
    let merge_radius = avg_d * ratio;

    // Greedy merging
    let mut merged = vec![false; n];
    let mut remap = vec![0usize; n];
    let mut out_points = Points::<f64>::new();

    for i in 0..n {
        if merged[i] {
            continue;
        }
        let nbrs = tree.find_within_radius(pts[i], merge_radius);
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let mut cnt = 0;
        for &(j, _) in &nbrs {
            if !merged[j] {
                cx += pts[j][0];
                cy += pts[j][1];
                cz += pts[j][2];
                cnt += 1;
            }
        }
        let idx = out_points.len();
        if cnt > 0 {
            out_points.push([cx / cnt as f64, cy / cnt as f64, cz / cnt as f64]);
        } else {
            out_points.push(pts[i]);
        }

        for &(j, _) in &nbrs {
            if !merged[j] {
                remap[j] = idx;
                merged[j] = true;
            }
        }
    }

    let mut out_polys = CellArray::new();
    for cell in input.polys.iter() {
        let mapped: Vec<i64> = cell.iter().map(|&id| remap[id as usize] as i64).collect();
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= 3 {
            out_polys.push_cell(&mapped);
        }
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
    fn reduces_points() {
        let mut pd = PolyData::new();
        for j in 0..10 {
            for i in 0..10 {
                pd.points.push([i as f64 * 0.1, j as f64 * 0.1, 0.0]);
            }
        }
        for j in 0..9 {
            for i in 0..9 {
                let a = (j * 10 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 11]);
                pd.polys.push_cell(&[a, a + 11, a + 10]);
            }
        }

        let result = coarsen(&pd, 25);
        assert!(result.points.len() < 100);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn already_small() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = coarsen(&pd, 10);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = coarsen(&pd, 5);
        assert_eq!(result.points.len(), 0);
    }
}
