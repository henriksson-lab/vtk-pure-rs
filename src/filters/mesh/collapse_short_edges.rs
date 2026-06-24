use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Collapse edges shorter than a threshold using k-d tree for efficiency.
///
/// Unlike `collapse_edges`, this uses a k-d tree to find merge candidates
/// efficiently. Produces a cleaner mesh with better degenerate handling.
pub fn collapse_short_edges_kdtree(input: &PolyData, min_length: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = crate::data::KdTree::build(&pts);
    let d2 = min_length * min_length;

    // Build merge map
    let mut remap = vec![usize::MAX; n];
    let mut out_pts = Points::<f64>::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let idx = out_pts.len();
        let nbrs = tree.find_within_radius(pts[i], min_length);

        // Compute centroid of cluster
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let mut cnt = 0;
        for &(j, jd2) in &nbrs {
            if remap[j] == usize::MAX && jd2 <= d2 {
                cx += pts[j][0];
                cy += pts[j][1];
                cz += pts[j][2];
                cnt += 1;
                remap[j] = idx;
            }
        }
        if cnt > 0 {
            out_pts.push([cx / cnt as f64, cy / cnt as f64, cz / cnt as f64]);
        }
    }

    // Remap cells
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
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_close_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[1, 2, 3]);

        let result = collapse_short_edges_kdtree(&pd, 0.01);
        assert!(result.points.len() < 4);
    }

    #[test]
    fn preserves_well_spaced() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = collapse_short_edges_kdtree(&pd, 0.001);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(collapse_short_edges_kdtree(&pd, 0.1).points.len(), 0);
    }
}
