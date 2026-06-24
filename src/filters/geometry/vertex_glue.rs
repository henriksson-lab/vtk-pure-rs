use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Merge coincident vertices within a tolerance.
///
/// Unlike `clean` which uses a spatial hash, this filter uses exact
/// grid-based snapping for deterministic results. Points are snapped
/// to a grid with cell size = tolerance, then merged if they land in
/// the same grid cell. Cell connectivity is updated and degenerate
/// cells are removed.
pub fn vertex_glue(input: &PolyData, tolerance: f64) -> PolyData {
    let tol = tolerance.max(1e-15);
    let inv_tol = 1.0 / tol;
    let n = input.points.len();

    // Snap each point to a grid cell and build a mapping
    let mut grid_map: HashMap<(i64, i64, i64), usize> = HashMap::new();
    let mut point_remap = vec![0usize; n];
    let mut out_points = Points::<f64>::new();

    for i in 0..n {
        let p = input.points.get(i);
        let gx = (p[0] * inv_tol).round() as i64;
        let gy = (p[1] * inv_tol).round() as i64;
        let gz = (p[2] * inv_tol).round() as i64;
        let key = (gx, gy, gz);

        let out_idx = if let Some(&idx) = grid_map.get(&key) {
            idx
        } else {
            let idx = out_points.len();
            out_points.push(p);
            grid_map.insert(key, idx);
            idx
        };
        point_remap[i] = out_idx;
    }

    // Remap polys
    let mut out_polys = CellArray::new();
    for cell in input.polys.iter() {
        let mapped: Vec<i64> = cell
            .iter()
            .map(|&id| point_remap[id as usize] as i64)
            .collect();
        // Remove degenerate cells
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= 3 {
            out_polys.push_cell(&mapped);
        }
    }

    // Remap lines
    let mut out_lines = CellArray::new();
    for cell in input.lines.iter() {
        let mapped: Vec<i64> = cell
            .iter()
            .map(|&id| point_remap[id as usize] as i64)
            .collect();
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= 2 {
            out_lines.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_coincident() {
        let mut pd = PolyData::new();
        // Two triangles sharing an edge but with duplicate points
        pd.points.push([0.0, 0.0, 0.0]); // 0
        pd.points.push([1.0, 0.0, 0.0]); // 1
        pd.points.push([0.5, 1.0, 0.0]); // 2
        pd.points.push([1.0, 0.0, 0.0]); // 3 = duplicate of 1
        pd.points.push([2.0, 0.0, 0.0]); // 4
        pd.points.push([0.5, 1.0, 0.0]); // 5 = duplicate of 2
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = vertex_glue(&pd, 0.01);
        assert_eq!(result.points.len(), 4); // 6 -> 4 (2 duplicates merged)
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn no_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertex_glue(&pd, 0.01);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn degenerate_removed() {
        let mut pd = PolyData::new();
        // Triangle where all 3 points are the same
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([0.0, 0.001, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertex_glue(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 0); // degenerate
    }

    #[test]
    fn large_tolerance() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);

        // Very large tolerance merges points 0 and 1
        let result = vertex_glue(&pd, 1.0);
        assert!(result.points.len() <= 3);
    }
}
