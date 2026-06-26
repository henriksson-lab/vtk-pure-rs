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

    // Bucket points by tolerance-sized grid cell, then check neighboring
    // buckets by actual distance so points within tolerance are not separated
    // just because they lie on opposite sides of a bucket boundary.
    let mut buckets: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    let mut point_remap = vec![0usize; n];
    let mut out_points = Points::<f64>::new();
    let tol2 = tol * tol;

    for i in 0..n {
        let p = input.points.get(i);
        let gx = (p[0] * inv_tol).floor() as i64;
        let gy = (p[1] * inv_tol).floor() as i64;
        let gz = (p[2] * inv_tol).floor() as i64;
        let key = (gx, gy, gz);

        let out_idx = if let Some(idx) = find_existing_point(p, key, &buckets, &out_points, tol2) {
            idx
        } else {
            let idx = out_points.len();
            out_points.push(p);
            buckets.entry(key).or_default().push(idx);
            idx
        };
        point_remap[i] = out_idx;
    }

    // Remap verts
    let mut out_verts = CellArray::new();
    for cell in input.verts.iter() {
        if let Some(mapped) = remap_cell(cell, &point_remap) {
            let mut unique = mapped.clone();
            unique.sort_unstable();
            unique.dedup();
            if !unique.is_empty() {
                out_verts.push_cell(&mapped);
            }
        }
    }

    // Remap polys
    let mut out_polys = CellArray::new();
    for cell in input.polys.iter() {
        if let Some(mapped) = remap_cell(cell, &point_remap) {
            // Remove degenerate cells
            let mut unique = mapped.clone();
            unique.sort_unstable();
            unique.dedup();
            if unique.len() >= 3 {
                out_polys.push_cell(&mapped);
            }
        }
    }

    // Remap lines
    let mut out_lines = CellArray::new();
    for cell in input.lines.iter() {
        if let Some(mapped) = remap_cell(cell, &point_remap) {
            let mut unique = mapped.clone();
            unique.sort_unstable();
            unique.dedup();
            if unique.len() >= 2 {
                out_lines.push_cell(&mapped);
            }
        }
    }

    // Remap triangle strips
    let mut out_strips = CellArray::new();
    for cell in input.strips.iter() {
        if let Some(mapped) = remap_cell(cell, &point_remap) {
            let mut unique = mapped.clone();
            unique.sort_unstable();
            unique.dedup();
            if unique.len() >= 3 {
                out_strips.push_cell(&mapped);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd.strips = out_strips;
    pd
}

fn remap_cell(cell: &[i64], point_remap: &[usize]) -> Option<Vec<i64>> {
    let mut mapped = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || (id as usize) >= point_remap.len() {
            return None;
        }
        mapped.push(point_remap[id as usize] as i64);
    }
    Some(mapped)
}

fn find_existing_point(
    p: [f64; 3],
    key: (i64, i64, i64),
    buckets: &HashMap<(i64, i64, i64), Vec<usize>>,
    out_points: &Points<f64>,
    tol2: f64,
) -> Option<usize> {
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let neighbor_key = (key.0 + dx, key.1 + dy, key.2 + dz);
                let Some(candidates) = buckets.get(&neighbor_key) else {
                    continue;
                };
                for &candidate in candidates {
                    let q = out_points.get(candidate);
                    let d2 = (p[0] - q[0]) * (p[0] - q[0])
                        + (p[1] - q[1]) * (p[1] - q[1])
                        + (p[2] - q[2]) * (p[2] - q[2]);
                    if d2 <= tol2 {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    None
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

    #[test]
    fn merges_across_bucket_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.49, 0.0, 0.0]);
        pd.points.push([0.51, 0.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.verts.push_cell(&[1]);

        let result = vertex_glue(&pd, 0.05);
        assert_eq!(result.points.len(), 1);
    }
}
