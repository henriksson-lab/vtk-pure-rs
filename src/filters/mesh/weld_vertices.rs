use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Weld (merge) vertices that are within a given distance tolerance.
///
/// Vertices closer than `tolerance` are merged into one. Cell connectivity is
/// updated to reference the merged vertices. Degenerate cells (with fewer than
/// 3 unique vertices for polygons) are discarded.
pub fn weld_vertices(input: &PolyData, tolerance: f64) -> PolyData {
    let n: usize = input.points.len();
    let tol2: f64 = tolerance * tolerance;

    // Spatial hashing for efficient neighbor lookup
    let inv_tol: f64 = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1e12
    };

    let mut new_points: Points<f64> = Points::new();
    let mut point_map: Vec<usize> = vec![0; n];
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();

    for i in 0..n {
        let p = input.points.get(i);
        let key = (
            (p[0] * inv_tol).round() as i64,
            (p[1] * inv_tol).round() as i64,
            (p[2] * inv_tol).round() as i64,
        );

        let mut found: Option<usize> = None;

        // Check 3x3x3 neighborhood
        'search: for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                for dz in -1i64..=1 {
                    let nkey = (key.0 + dx, key.1 + dy, key.2 + dz);
                    if let Some(indices) = grid.get(&nkey) {
                        for &idx in indices {
                            let q = new_points.get(idx);
                            let d2: f64 = (p[0] - q[0]) * (p[0] - q[0])
                                + (p[1] - q[1]) * (p[1] - q[1])
                                + (p[2] - q[2]) * (p[2] - q[2]);
                            if d2 <= tol2 {
                                found = Some(idx);
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        match found {
            Some(idx) => {
                point_map[i] = idx;
            }
            None => {
                let new_idx: usize = new_points.len();
                new_points.push(p);
                point_map[i] = new_idx;
                grid.entry(key).or_default().push(new_idx);
            }
        }
    }

    let mut output = PolyData::new();
    output.points = new_points;

    // Remap cell arrays
    output.polys = remap_cell_array(&input.polys, &point_map, 3);
    output.verts = remap_cell_array(&input.verts, &point_map, 1);
    output.lines = remap_cell_array(&input.lines, &point_map, 2);
    output.strips = remap_cell_array(&input.strips, &point_map, 3);

    output
}

/// Remap cell indices and discard degenerate cells.
fn remap_cell_array(cells: &CellArray, point_map: &[usize], min_unique: usize) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells.iter() {
        let mapped: Vec<i64> = cell
            .iter()
            .map(|&id| point_map[id as usize] as i64)
            .collect();

        // Check for enough unique vertices
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= min_unique {
            out.push_cell(&mapped);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weld_duplicate_vertices() {
        // Two triangles sharing an edge, but with duplicated vertices
        let pts: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [0.5, 1.0, 0.0], // 2
            [1.0, 0.0, 0.0], // 3 = duplicate of 1
            [2.0, 0.0, 0.0], // 4
            [1.5, 1.0, 0.0], // 5
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2], [3, 4, 5]];
        let pd = PolyData::from_triangles(pts, tris);

        let result = weld_vertices(&pd, 1e-6);
        // Should have 5 unique points (vertex 3 merged with vertex 1)
        assert_eq!(result.points.len(), 5);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn weld_with_tolerance() {
        // Points that are close but not identical
        let pts: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.001, 0.0, 0.0], // close to point 1
            [2.0, 0.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2], [3, 4, 5]];
        let pd = PolyData::from_triangles(pts, tris);

        // With a large enough tolerance, point 3 merges with point 1
        let result = weld_vertices(&pd, 0.01);
        assert_eq!(result.points.len(), 5);

        // With a tiny tolerance, nothing merges
        let result2 = weld_vertices(&pd, 1e-6);
        assert_eq!(result2.points.len(), 6);
    }

    #[test]
    fn degenerate_cells_removed() {
        // A triangle where two vertices are the same after welding
        let pts: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate of 0
            [1.0, 0.0, 0.0],
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2]];
        let pd = PolyData::from_triangles(pts, tris);

        let result = weld_vertices(&pd, 1e-6);
        // The triangle becomes degenerate (only 2 unique vertices)
        assert_eq!(result.polys.num_cells(), 0);
    }
}
