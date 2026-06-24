//! StaticCleanPolyData — remove degenerate cells and duplicate points using spatial hashing.

use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

fn remap_and_filter_cells(
    ca: &CellArray,
    remap: &[usize],
    pts: &Points<f64>,
    min_area: f64,
    check_area: bool,
) -> CellArray {
    let mut out = CellArray::new();
    for cell in ca.iter() {
        let mapped: Vec<i64> = cell.iter().map(|&v| remap[v as usize] as i64).collect();
        let mut unique: Vec<i64> = mapped.clone();
        unique.sort_unstable();
        unique.dedup();
        if unique.len() < 2 {
            continue;
        }
        if check_area && mapped.len() >= 3 {
            let a = pts.get(mapped[0] as usize);
            let b = pts.get(mapped[1] as usize);
            let c = pts.get(mapped[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
            if area < min_area {
                continue;
            }
        }
        out.push_cell(&mapped);
    }
    out
}

/// Remove degenerate cells (zero-area triangles) and merge duplicate points
/// using spatial hashing, without changing topology.
///
/// `tolerance` controls the merge distance for duplicate points and the
/// minimum triangle area for degenerate detection.
pub fn static_clean_poly_data(input: &PolyData, tolerance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // --- Phase 1: merge duplicate points via spatial hashing ---
    let inv_cell = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1.0e10
    };
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    let mut remap = vec![0usize; n];
    let mut new_pts = Points::<f64>::new();
    let mut new_index: Vec<usize> = Vec::new(); // original index of each new point

    for i in 0..n {
        let p = input.points.get(i);
        let gx = (p[0] * inv_cell).floor() as i64;
        let gy = (p[1] * inv_cell).floor() as i64;
        let gz = (p[2] * inv_cell).floor() as i64;

        let tol2 = tolerance * tolerance;
        let mut found = None;

        // Search neighboring cells (3x3x3)
        'outer: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(bucket) = grid.get(&(gx + dx, gy + dy, gz + dz)) {
                        for &ni in bucket {
                            let q = new_pts.get(ni);
                            let d2 = (p[0] - q[0]).powi(2)
                                + (p[1] - q[1]).powi(2)
                                + (p[2] - q[2]).powi(2);
                            if d2 < tol2 {
                                found = Some(ni);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if let Some(ni) = found {
            remap[i] = ni;
        } else {
            let ni = new_pts.len();
            new_pts.push(p);
            new_index.push(i);
            remap[i] = ni;
            grid.entry((gx, gy, gz)).or_default().push(ni);
        }
    }

    // --- Phase 2: remap cells and remove degenerate triangles ---
    let min_area = tolerance;

    let polys = remap_and_filter_cells(&input.polys, &remap, &new_pts, min_area, true);
    let lines = remap_and_filter_cells(&input.lines, &remap, &new_pts, min_area, false);
    let verts = remap_and_filter_cells(&input.verts, &remap, &new_pts, min_area, false);

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result.lines = lines;
    result.verts = verts;
    let _ = new_index;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_degenerate_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        // Degenerate: all three points nearly identical
        pd.points.push([5.0, 5.0, 5.0]);
        pd.points.push([5.0, 5.0, 5.0]);
        pd.points.push([5.0, 5.0, 5.0]);

        pd.polys.push_cell(&[0, 1, 2]); // good triangle
        pd.polys.push_cell(&[3, 4, 5]); // degenerate

        let result = static_clean_poly_data(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn merges_duplicate_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.001]); // near-duplicate of point 0

        pd.polys.push_cell(&[0, 1, 2]);

        let result = static_clean_poly_data(&pd, 0.01);
        // Point 3 should be merged with point 0
        assert!(result.points.len() <= 3);
    }

    #[test]
    fn preserves_valid_mesh() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = static_clean_poly_data(&pd, 1e-8);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }
}
