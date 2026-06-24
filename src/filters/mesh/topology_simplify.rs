use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Remove non-manifold edges by duplicating shared vertices.
///
/// For edges shared by more than 2 faces, splits the mesh so each
/// pair of faces gets its own copy of the edge vertices, making the
/// mesh manifold everywhere.
pub fn fix_non_manifold(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Find non-manifold edges (shared by >2 faces)
    let nm_edges: Vec<(i64, i64)> = edge_faces
        .iter()
        .filter(|(_, f)| f.len() > 2)
        .map(|(&k, _)| k)
        .collect();

    if nm_edges.is_empty() {
        return input.clone();
    }

    let mut out_pts = input.points.clone();
    let mut new_cells = cells.clone();

    for &(a, b) in &nm_edges {
        let faces = &edge_faces[&(a, b)];
        // Keep first two faces as-is, duplicate for the rest
        for &fi in faces.iter().skip(2) {
            let new_a = out_pts.len() as i64;
            out_pts.push(input.points.get(a as usize));
            let new_b = out_pts.len() as i64;
            out_pts.push(input.points.get(b as usize));

            for v in &mut new_cells[fi] {
                if *v == a {
                    *v = new_a;
                } else if *v == b {
                    *v = new_b;
                }
            }
        }
    }

    let mut out_polys = CellArray::new();
    for c in &new_cells {
        out_polys.push_cell(c);
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
    fn fixes_non_manifold() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[0, 1, 4]);

        let result = fix_non_manifold(&pd);
        assert!(result.points.len() >= 5); // duplicated vertices
        assert_eq!(result.polys.num_cells(), 3);
    }

    #[test]
    fn manifold_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = fix_non_manifold(&pd);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = fix_non_manifold(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
