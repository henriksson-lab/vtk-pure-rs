//! Convert independent triangles to triangle strips for efficient rendering.

use crate::data::{CellArray, PolyData};

/// Convert triangles to triangle strips using a greedy algorithm.
pub fn build_triangle_strips(mesh: &PolyData) -> PolyData {
    let num_cells = mesh.polys.num_cells();
    if num_cells == 0 {
        return mesh.clone();
    }

    // Build adjacency: for each edge, which triangles share it
    let mut edge_to_tris: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    for (ci, cell) in cells.iter().enumerate() {
        if cell.len() != 3 {
            continue;
        }
        for i in 0..3 {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % 3] as usize;
            let key = (a.min(b), a.max(b));
            edge_to_tris.entry(key).or_default().push(ci);
        }
    }

    let mut used = vec![false; num_cells];
    let mut strips = CellArray::new();
    let mut remaining_polys = CellArray::new();

    for start in 0..num_cells {
        if used[start] || cells[start].len() != 3 {
            continue;
        }
        used[start] = true;
        let mut strip: Vec<i64> = cells[start].clone();

        // Try to extend forward
        loop {
            let last_edge = if strip.len() % 2 == 1 {
                (
                    strip[strip.len() - 2] as usize,
                    strip[strip.len() - 1] as usize,
                )
            } else {
                (
                    strip[strip.len() - 1] as usize,
                    strip[strip.len() - 2] as usize,
                )
            };
            let key = (last_edge.0.min(last_edge.1), last_edge.0.max(last_edge.1));
            let next = edge_to_tris
                .get(&key)
                .and_then(|tris| tris.iter().find(|&&t| !used[t] && cells[t].len() == 3))
                .copied();
            match next {
                Some(ti) => {
                    used[ti] = true;
                    // Find the vertex not on the shared edge
                    let c = &cells[ti];
                    let new_v = c
                        .iter()
                        .find(|&&v| v as usize != key.0 && v as usize != key.1);
                    match new_v {
                        Some(&v) => strip.push(v),
                        None => break,
                    }
                }
                None => break,
            }
        }

        if strip.len() >= 3 {
            strips.push_cell(&strip);
        }
    }

    // Collect non-triangle cells
    for (ci, cell) in cells.iter().enumerate() {
        if cell.len() != 3 && !used[ci] {
            remaining_polys.push_cell(cell);
        }
    }

    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.strips = strips;
    result.polys = remaining_polys;
    result
}

/// Count how many triangles the strips represent.
pub fn count_strip_triangles(mesh: &PolyData) -> usize {
    mesh.strips
        .iter()
        .map(|s| if s.len() >= 3 { s.len() - 2 } else { 0 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_strip_build() {
        // Two adjacent triangles sharing edge 1-2
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = build_triangle_strips(&mesh);
        let total = count_strip_triangles(&result);
        assert_eq!(total, 2);
    }
    #[test]
    fn test_single_tri() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = build_triangle_strips(&mesh);
        assert_eq!(count_strip_triangles(&result), 1);
    }
}
