use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Midpoint subdivision: split each triangle into 4 by inserting
/// midpoints on each edge.
///
/// This is simpler than Loop subdivision — it doesn't reposition
/// existing vertices, just splits edges at their midpoints.
/// Good for increasing mesh resolution without smoothing.
pub fn subdivide_midpoint(input: &PolyData) -> PolyData {
    let mut out_points = input.points.clone();
    let mut out_polys = CellArray::new();

    // Cache midpoints to avoid duplicates on shared edges
    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

    let get_midpoint =
        |a: i64, b: i64, pts: &mut Points<f64>, cache: &mut HashMap<(i64, i64), i64>| -> i64 {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&mid) = cache.get(&key) {
                return mid;
            }
            let pa = pts.get(a as usize);
            let pb = pts.get(b as usize);
            let idx = pts.len() as i64;
            pts.push([
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ]);
            cache.insert(key, idx);
            idx
        };

    for cell in input.polys.iter() {
        if cell.len() == 3 {
            let a = cell[0];
            let b = cell[1];
            let c = cell[2];

            let ab = get_midpoint(a, b, &mut out_points, &mut midpoint_cache);
            let bc = get_midpoint(b, c, &mut out_points, &mut midpoint_cache);
            let ca = get_midpoint(c, a, &mut out_points, &mut midpoint_cache);

            out_polys.push_cell(&[a, ab, ca]);
            out_polys.push_cell(&[ab, b, bc]);
            out_polys.push_cell(&[ca, bc, c]);
            out_polys.push_cell(&[ab, bc, ca]);
        } else if cell.len() == 4 {
            // Quad: split into 4 quads
            let a = cell[0];
            let b = cell[1];
            let c = cell[2];
            let d = cell[3];

            let ab = get_midpoint(a, b, &mut out_points, &mut midpoint_cache);
            let bc = get_midpoint(b, c, &mut out_points, &mut midpoint_cache);
            let cd = get_midpoint(c, d, &mut out_points, &mut midpoint_cache);
            let da = get_midpoint(d, a, &mut out_points, &mut midpoint_cache);

            // Center point
            let center = out_points.len() as i64;
            let pa = input.points.get(a as usize);
            let pc = input.points.get(c as usize);
            out_points.push([
                (pa[0] + pc[0]) * 0.5,
                (pa[1] + pc[1]) * 0.5,
                (pa[2] + pc[2]) * 0.5,
            ]);

            out_polys.push_cell(&[a, ab, center, da]);
            out_polys.push_cell(&[ab, b, bc, center]);
            out_polys.push_cell(&[center, bc, c, cd]);
            out_polys.push_cell(&[da, center, cd, d]);
        } else {
            out_polys.push_cell(cell);
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
    fn subdivide_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_midpoint(&pd);
        // 3 original + 3 midpoints = 6 points
        assert_eq!(result.points.len(), 6);
        // 4 sub-triangles
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn shared_edge_reuses_midpoint() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = subdivide_midpoint(&pd);
        // 4 original + 5 midpoints (shared edge 1-2 produces 1 midpoint) = 9
        assert_eq!(result.points.len(), 9);
        assert_eq!(result.polys.num_cells(), 8);
    }

    #[test]
    fn subdivide_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = subdivide_midpoint(&pd);
        // 4 corners + 4 edge midpoints + 1 center = 9
        assert_eq!(result.points.len(), 9);
        // 4 sub-quads
        assert_eq!(result.polys.num_cells(), 4);
    }
}
