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

    for strip in input.strips.iter() {
        strips.push_cell(strip);
    }

    // Greedy strip building
    for start_fi in 0..nc {
        if used[start_fi] {
            continue;
        }
        let strip = best_strip_from(start_fi, &cells, &edge_faces, &mut used);

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

    let mut pd = input.clone();
    pd.strips = strips;
    pd.polys = non_tri_polys;
    pd
}

fn best_strip_from(
    start_fi: usize,
    cells: &[Vec<i64>],
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    used: &mut [bool],
) -> Vec<i64> {
    let tri = &cells[start_fi];
    let seeds = [
        vec![tri[0], tri[1], tri[2]],
        vec![tri[1], tri[2], tri[0]],
        vec![tri[2], tri[0], tri[1]],
        vec![tri[2], tri[1], tri[0]],
        vec![tri[1], tri[0], tri[2]],
        vec![tri[0], tri[2], tri[1]],
    ];

    let mut best_strip = seeds[0].clone();
    let mut best_used = used.to_vec();
    best_used[start_fi] = true;

    for seed in seeds {
        let mut trial_used = used.to_vec();
        trial_used[start_fi] = true;
        let mut strip = seed;

        loop {
            let last_edge_a = strip[strip.len() - 2];
            let last_edge_b = strip[strip.len() - 1];
            let key = if last_edge_a < last_edge_b {
                (last_edge_a, last_edge_b)
            } else {
                (last_edge_b, last_edge_a)
            };

            let Some(adj) = edge_faces.get(&key) else {
                break;
            };
            let mut next_vertex = None;
            for &fi in adj {
                if trial_used[fi] {
                    continue;
                }
                if let Some(&v) = cells[fi]
                    .iter()
                    .find(|&&v| v != last_edge_a && v != last_edge_b)
                {
                    next_vertex = Some((fi, v));
                    break;
                }
            }
            let Some((fi, v)) = next_vertex else {
                break;
            };
            trial_used[fi] = true;
            strip.push(v);
        }

        if strip.len() > best_strip.len() {
            best_strip = strip;
            best_used = trial_used;
        }
    }

    used.copy_from_slice(&best_used);
    best_strip
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
        assert!(result.strips.iter().any(|strip| strip.len() == 4));
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

    #[test]
    fn preserves_lines_and_existing_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
    }
}
