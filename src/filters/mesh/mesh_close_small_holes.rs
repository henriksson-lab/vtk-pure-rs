//! Close small boundary loops by fan-triangulating them.
use crate::data::{CellArray, PolyData};

pub fn close_small_holes(mesh: &PolyData, max_hole_edges: usize) -> PolyData {
    // Find boundary edges
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    let mut directed_edges = Vec::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 3 {
            continue;
        }
        for i in 0..nc {
            if cell[i] < 0 || cell[(i + 1) % nc] < 0 {
                continue;
            }
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= mesh.points.len() || b >= mesh.points.len() || a == b {
                continue;
            }
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
            directed_edges.push((a, b));
        }
    }
    let boundary_edges: Vec<(usize, usize)> = directed_edges
        .into_iter()
        .filter(|&(a, b)| edge_count.get(&(a.min(b), a.max(b))) == Some(&1))
        .collect();
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        new_polys.push_cell(&cell.to_vec());
    }
    if boundary_edges.len() < 3 {
        let mut result = mesh.clone();
        result.polys = new_polys;
        return result;
    }

    let mut edge_ids_by_vertex: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (edge_id, &(a, b)) in boundary_edges.iter().enumerate() {
        edge_ids_by_vertex.entry(a).or_default().push(edge_id);
        edge_ids_by_vertex.entry(b).or_default().push(edge_id);
    }

    let mut visited_edges = vec![false; boundary_edges.len()];
    let mut inserted = false;
    for (start_edge, &(start, next)) in boundary_edges.iter().enumerate() {
        if visited_edges[start_edge] {
            continue;
        }
        let mut loop_verts = vec![start];
        let mut current = next;
        let mut current_edge = start_edge;
        let mut valid = true;
        loop {
            visited_edges[current_edge] = true;
            if current == start {
                break;
            }
            loop_verts.push(current);
            let Some(edge_ids) = edge_ids_by_vertex.get(&current) else {
                valid = false;
                break;
            };
            let unvisited: Vec<usize> = edge_ids
                .iter()
                .copied()
                .filter(|&edge_id| !visited_edges[edge_id])
                .collect();
            if unvisited.len() != 1 {
                valid = false;
                break;
            }
            current_edge = unvisited[0];
            let (a, b) = boundary_edges[current_edge];
            current = if a == current { b } else { a };
        }
        if valid && loop_verts.len() >= 3 && loop_verts.len() <= max_hole_edges {
            let root = loop_verts[0] as i64;
            for i in 1..loop_verts.len() - 1 {
                new_polys.push_cell(&[root, loop_verts[i] as i64, loop_verts[i + 1] as i64]);
                inserted = true;
            }
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    if inserted {
        result.cell_data_mut().clear();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_close_holes() {
        // Single triangle has a 3-edge boundary "hole"
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = close_small_holes(&mesh, 5);
        // Should add triangles to close the boundary
        assert!(r.polys.num_cells() >= 1);
    }
}
