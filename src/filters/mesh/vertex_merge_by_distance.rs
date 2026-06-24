use crate::data::{CellArray, KdTree, Points, PolyData};
use std::collections::HashMap;

/// Merge vertices that are within `distance` of each other using k-d tree.
///
/// More efficient than `vertex_glue` for large meshes. Merges to the
/// first-encountered vertex in each cluster.
pub fn merge_close_vertices(input: &PolyData, distance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);
    let d2 = distance * distance;

    let mut remap = vec![usize::MAX; n];
    let mut out_pts = Points::<f64>::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let idx = out_pts.len();
        out_pts.push(pts[i]);
        remap[i] = idx;

        let nbrs = tree.find_within_radius(pts[i], distance);
        for &(j, jd2) in &nbrs {
            if j != i && remap[j] == usize::MAX && jd2 <= d2 {
                remap[j] = idx;
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

    let mut out_lines = CellArray::new();
    for cell in input.lines.iter() {
        let mapped: Vec<i64> = cell.iter().map(|&id| remap[id as usize] as i64).collect();
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= 2 {
            out_lines.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]); // close to 0
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.001, 0.0, 0.0]); // close to 2
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 2, 4]);
        pd.polys.push_cell(&[1, 3, 4]);

        let result = merge_close_vertices(&pd, 0.01);
        assert!(result.points.len() < 5);
    }

    #[test]
    fn no_merge_far() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = merge_close_vertices(&pd, 0.001);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(merge_close_vertices(&pd, 0.1).points.len(), 0);
    }
}
