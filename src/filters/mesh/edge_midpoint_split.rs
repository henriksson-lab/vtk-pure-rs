use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Split every triangle at edge midpoints, producing 4 triangles per input triangle.
///
/// This is a simple midpoint subdivision: each edge is split at its midpoint
/// and the original triangle is replaced by 4 smaller triangles. Original
/// vertex positions are not moved (unlike Loop subdivision).
///
/// Only polygon cells with exactly 3 vertices (triangles) are subdivided.
/// Non-triangle cells are dropped.
pub fn midpoint_split(input: &PolyData) -> PolyData {
    let mut out_points: Points<f64> = Points::new();

    // Copy original points
    for i in 0..input.points.len() {
        out_points.push(input.points.get(i));
    }

    // Cache for midpoints: (min_id, max_id) -> new point index
    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

    let mut get_midpoint = |points: &mut Points<f64>,
                            cache: &mut HashMap<(usize, usize), usize>,
                            a: usize,
                            b: usize|
     -> usize {
        let key: (usize, usize) = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = cache.get(&key) {
            return idx;
        }
        let pa = input.points.get(a);
        let pb = input.points.get(b);
        let mid: [f64; 3] = [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ];
        let idx: usize = points.len();
        points.push(mid);
        cache.insert(key, idx);
        idx
    };

    let mut out_polys: CellArray = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let v0: usize = cell[0] as usize;
        let v1: usize = cell[1] as usize;
        let v2: usize = cell[2] as usize;

        let m01: usize = get_midpoint(&mut out_points, &mut midpoint_cache, v0, v1);
        let m12: usize = get_midpoint(&mut out_points, &mut midpoint_cache, v1, v2);
        let m20: usize = get_midpoint(&mut out_points, &mut midpoint_cache, v2, v0);

        // 4 sub-triangles
        out_polys.push_cell(&[v0 as i64, m01 as i64, m20 as i64]);
        out_polys.push_cell(&[v1 as i64, m12 as i64, m01 as i64]);
        out_polys.push_cell(&[v2 as i64, m20 as i64, m12 as i64]);
        out_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
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
    fn single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 4);
        // 3 original + 3 midpoints = 6 points
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn two_triangles_shared_edge() {
        // Two triangles sharing edge 1-2; midpoint on that edge should be shared
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],  // 0
                [2.0, 0.0, 0.0],  // 1
                [1.0, 2.0, 0.0],  // 2
                [1.0, -2.0, 0.0], // 3
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 8); // 4 per triangle
                                                 // 4 original + 5 unique midpoints (edge 0-1 shared) = 9
                                                 // edges: 0-1, 1-2, 2-0, 0-3, 3-1 => 5 midpoints
        assert_eq!(result.points.len(), 9);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }
}
