//! Orient mesh faces consistently (outward normals).

use crate::data::{CellArray, PolyData};

/// Orient all faces consistently using BFS face propagation.
pub fn orient_faces_consistent(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return mesh.clone();
    }

    // Build edge-face adjacency
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut oriented = vec![false; nc];
    let mut flipped = vec![false; nc];
    let mut queue = std::collections::VecDeque::new();
    oriented[0] = true;
    queue.push_back(0);

    while let Some(ci) = queue.pop_front() {
        let cell = &cells[ci];
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let key = (a.min(b), a.max(b));
            if let Some(neighbors) = edge_faces.get(&key) {
                for &ni in neighbors {
                    if oriented[ni] {
                        continue;
                    }
                    oriented[ni] = true;
                    // Check if neighbor has same edge direction (needs flip)
                    let ncell = &cells[ni];
                    let edge_dir_same = has_same_edge_direction(ncell, a, b);
                    let parent_flipped = flipped[ci];
                    // If parent is flipped, its effective edge is reversed
                    let need_flip = edge_dir_same ^ parent_flipped;
                    flipped[ni] = need_flip;
                    queue.push_back(ni);
                }
            }
        }
    }

    let mut new_polys = CellArray::new();
    for (ci, cell) in cells.iter().enumerate() {
        if flipped[ci] {
            let mut r = cell.clone();
            r.reverse();
            new_polys.push_cell(&r);
        } else {
            new_polys.push_cell(cell);
        }
    }

    let mut result = mesh.clone();
    result.polys = new_polys;
    result
}

fn has_same_edge_direction(cell: &[i64], a: usize, b: usize) -> bool {
    let n = cell.len();
    for i in 0..n {
        if cell[i] as usize == a && cell[(i + 1) % n] as usize == b {
            return true;
        }
    }
    false
}

/// Check if all faces have consistent winding.
pub fn is_consistently_oriented(mesh: &PolyData) -> bool {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut edge_dirs: std::collections::HashMap<(usize, usize), Vec<bool>> =
        std::collections::HashMap::new();
    for cell in &cells {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let key = (a.min(b), a.max(b));
            let forward = a < b;
            edge_dirs.entry(key).or_default().push(forward);
        }
    }
    // For consistent orientation, shared edges should have opposite directions
    edge_dirs.values().all(|dirs| {
        if dirs.len() != 2 {
            true
        }
        // boundary or non-manifold
        else {
            dirs[0] != dirs[1]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_orient() {
        // Two triangles with inconsistent winding
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.5, 1.0, 0.0]);
        mesh.points.push([1.5, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.polys.push_cell(&[1, 2, 3]); // same edge direction as first -> inconsistent
        let r = orient_faces_consistent(&mesh);
        assert!(is_consistently_oriented(&r));
    }
    #[test]
    fn test_already_consistent() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]], // edge 1-2 in opposite dirs -> consistent
        );
        assert!(is_consistently_oriented(&mesh));
    }
}
