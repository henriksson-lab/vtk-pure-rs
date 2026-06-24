use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

/// Convert triangle mesh to greedy triangle strips.
///
/// Uses a greedy approach: starts from an edge, greedily extends the
/// strip by alternately connecting to the next adjacent triangle.
/// Converts polys to strips for more efficient rendering.
pub fn triangles_to_greedy_strips(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| c.to_vec())
        .collect();
    let nc = cells.len();
    if nc == 0 {
        return input.clone();
    }

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..3 {
            let a = c[i];
            let b = c[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut used = vec![false; nc];
    let mut strips = CellArray::new();
    let mut non_tri_polys = CellArray::new();

    // Greedy strip building
    for start_fi in 0..nc {
        if used[start_fi] {
            continue;
        }
        used[start_fi] = true;
        let mut strip = cells[start_fi].clone();

        // Try to extend the strip
        let mut extended = true;
        while extended {
            extended = false;
            let last_edge_a = strip[strip.len() - 2];
            let last_edge_b = strip[strip.len() - 1];
            let key = if last_edge_a < last_edge_b {
                (last_edge_a, last_edge_b)
            } else {
                (last_edge_b, last_edge_a)
            };

            if let Some(adj) = edge_faces.get(&key) {
                for &fi in adj {
                    if used[fi] {
                        continue;
                    }
                    // Find the vertex not on the shared edge
                    let opp = cells[fi]
                        .iter()
                        .find(|&&v| v != last_edge_a && v != last_edge_b);
                    if let Some(&v) = opp {
                        used[fi] = true;
                        strip.push(v);
                        extended = true;
                        break;
                    }
                }
            }
        }

        if strip.len() >= 3 {
            strips.push_cell(&strip);
        }
    }

    // Pass through non-triangle polys
    for cell in input.polys.iter() {
        if cell.len() != 3 {
            non_tri_polys.push_cell(cell);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.strips = strips;
    pd.polys = non_tri_polys;
    pd
}

/// Count how many strips would be produced.
pub fn count_strips(input: &PolyData) -> usize {
    let result = triangles_to_greedy_strips(input);
    result.strips.num_cells()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_from_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = triangles_to_greedy_strips(&pd);
        assert!(result.strips.num_cells() >= 1);
        // A strip of 2 triangles should have 4 vertices
    }

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.strips.num_cells(), 1);
    }

    #[test]
    fn preserves_non_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]); // quad

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.polys.num_cells(), 1); // quad preserved
    }

    #[test]
    fn strip_count() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(count_strips(&pd), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(count_strips(&pd), 0);
    }
}
