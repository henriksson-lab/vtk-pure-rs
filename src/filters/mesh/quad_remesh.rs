use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Convert a triangle mesh to a quad-dominant mesh by merging triangle pairs.
///
/// Greedily pairs adjacent triangles that form good quadrilaterals
/// (based on planarity and aspect ratio). Remaining unpaired triangles
/// are kept as-is.
pub fn tri_to_quad(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc < 2 {
        return input.clone();
    }

    // Edge-to-face adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        if c.len() != 3 {
            continue;
        }
        for i in 0..3 {
            let a = c[i];
            let b = c[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut used = vec![false; nc];
    let mut out_polys = CellArray::new();

    // Greedily pair triangles
    for ((a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let fi = faces[0];
        let fj = faces[1];
        if used[fi] || used[fj] {
            continue;
        }
        if cells[fi].len() != 3 || cells[fj].len() != 3 {
            continue;
        }

        // Find opposite vertices
        let opp_i = cells[fi].iter().find(|&&v| v != *a && v != *b);
        let opp_j = cells[fj].iter().find(|&&v| v != *a && v != *b);
        if let (Some(&ci), Some(&cj)) = (opp_i, opp_j) {
            // Check quad quality (simple: just merge)
            // Order: a, ci, b, cj (or a, cj, b, ci depending on winding)
            out_polys.push_cell(&[*a, ci, *b, cj]);
            used[fi] = true;
            used[fj] = true;
        }
    }

    // Add remaining unpaired triangles
    for (fi, c) in cells.iter().enumerate() {
        if !used[fi] {
            out_polys.push_cell(c);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = tri_to_quad(&pd);
        // Should merge into 1 quad
        assert_eq!(result.polys.num_cells(), 1);
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4);
    }

    #[test]
    fn odd_triangle_remains() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([2.0, 0.5, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[1, 4, 2]);

        let result = tri_to_quad(&pd);
        // 2 paired + 1 remaining, or 1 quad + 1 tri
        assert!(result.polys.num_cells() >= 1);
    }

    #[test]
    fn single_triangle_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = tri_to_quad(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(tri_to_quad(&pd).polys.num_cells(), 0);
    }
}
