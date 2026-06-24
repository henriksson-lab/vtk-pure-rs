//! Extract boundary loops from a mesh as polylines.

use crate::data::{CellArray, Points, PolyData};

/// Extract boundary edges as polyline loops.
pub fn extract_boundary_loops(mesh: &PolyData) -> PolyData {
    // Find boundary edges (shared by exactly 1 face)
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let boundary_edges: Vec<(usize, usize)> = edge_count
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();

    if boundary_edges.is_empty() {
        let mut r = PolyData::new();
        r.points = Points::<f64>::new();
        return r;
    }

    // Build adjacency for boundary vertices
    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &boundary_edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    // Trace loops
    let mut visited_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    let mut loops: Vec<Vec<usize>> = Vec::new();

    for &start in adj.keys() {
        if adj[&start]
            .iter()
            .all(|&nb| visited_edges.contains(&(start.min(nb), start.max(nb))))
        {
            continue;
        }
        let mut loop_verts = vec![start];
        let mut current = start;
        loop {
            let next = adj
                .get(&current)
                .and_then(|nbs| {
                    nbs.iter()
                        .find(|&&nb| !visited_edges.contains(&(current.min(nb), current.max(nb))))
                })
                .copied();
            match next {
                Some(nb) => {
                    visited_edges.insert((current.min(nb), current.max(nb)));
                    if nb == start {
                        break;
                    }
                    loop_verts.push(nb);
                    current = nb;
                }
                None => break,
            }
        }
        if loop_verts.len() >= 2 {
            loops.push(loop_verts);
        }
    }

    // Build output
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for lp in &loops {
        let ids: Vec<i64> = lp
            .iter()
            .map(|&v| {
                *pt_map.entry(v).or_insert_with(|| {
                    let idx = pts.len();
                    pts.push(mesh.points.get(v));
                    idx
                }) as i64
            })
            .collect();
        lines.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Count number of boundary loops.
pub fn boundary_loop_count(mesh: &PolyData) -> usize {
    extract_boundary_loops(mesh).lines.num_cells()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_open_mesh() {
        // Single triangle has one boundary loop
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(boundary_loop_count(&mesh), 1);
    }
    #[test]
    fn test_two_tris() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert_eq!(boundary_loop_count(&mesh), 1);
    }
}
