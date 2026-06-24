//! Pre-render mesh refinement via midpoint subdivision.
//!
//! Each subdivision level splits every triangle into 4 triangles by
//! inserting midpoints along edges.

use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Configuration for subdivision surface rendering.
#[derive(Debug, Clone)]
pub struct SubdivisionConfig {
    /// Whether subdivision is enabled.
    pub enabled: bool,
    /// Number of subdivision levels to apply.
    pub level: u32,
}

impl Default for SubdivisionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 1,
        }
    }
}

/// Apply midpoint subdivision to a PolyData mesh.
///
/// Each triangle is split into 4 triangles by inserting midpoints at
/// edge centers. This is repeated `level` times.
pub fn subdivide_for_render(poly_data: &PolyData, level: u32) -> PolyData {
    if level == 0 {
        return poly_data.clone();
    }

    let mut current = poly_data.clone();

    for _ in 0..level {
        current = subdivide_once(&current);
    }

    current
}

fn subdivide_once(pd: &PolyData) -> PolyData {
    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();

    // Copy existing points
    for i in 0..pd.points.len() {
        new_points.push(pd.points.get(i));
    }

    // Map from edge (min_idx, max_idx) to midpoint index
    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

    let get_midpoint = |a: usize,
                        b: usize,
                        points: &mut Points<f64>,
                        cache: &mut HashMap<(usize, usize), usize>|
     -> usize {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = cache.get(&key) {
            return idx;
        }
        let pa = pd.points.get(a);
        let pb = pd.points.get(b);
        let mid = [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ];
        let idx = points.len();
        points.push(mid);
        cache.insert(key, idx);
        idx
    };

    for cell in pd.polys.iter() {
        if cell.len() == 3 {
            let a = cell[0] as usize;
            let b = cell[1] as usize;
            let c = cell[2] as usize;

            let ab = get_midpoint(a, b, &mut new_points, &mut midpoint_cache);
            let bc = get_midpoint(b, c, &mut new_points, &mut midpoint_cache);
            let ca = get_midpoint(c, a, &mut new_points, &mut midpoint_cache);

            // 4 sub-triangles
            new_polys.push_cell(&[a as i64, ab as i64, ca as i64]);
            new_polys.push_cell(&[ab as i64, b as i64, bc as i64]);
            new_polys.push_cell(&[ca as i64, bc as i64, c as i64]);
            new_polys.push_cell(&[ab as i64, bc as i64, ca as i64]);
        } else {
            // Non-triangle cells are passed through unchanged
            new_polys.push_cell(&cell.iter().copied().collect::<Vec<_>>());
        }
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn test_subdivide_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = subdivide_for_render(&pd, 1);
        // 1 triangle -> 4 triangles, 3 original + 3 midpoints = 6 points
        assert_eq!(result.polys.num_cells(), 4);
        assert_eq!(result.points.len(), 6);

        // Level 2: 4 triangles -> 16 triangles
        let result2 = subdivide_for_render(&pd, 2);
        assert_eq!(result2.polys.num_cells(), 16);
    }
}
