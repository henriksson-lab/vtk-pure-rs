//! Face orientation analysis and repair.

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Check face orientation consistency and add a "Consistent" cell data array.
///
/// 1.0 = consistent with neighbors, 0.0 = inconsistent.
pub fn check_face_orientation(mesh: &PolyData) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();

    // For each directed edge, track which face uses it
    let mut directed: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            directed.entry((a, b)).or_default().push(ci);
        }
    }

    let mut consistent = vec![1.0f64; n_cells];
    for (&(a, b), faces) in &directed {
        if faces.len() > 1 {
            // Same directed edge used by multiple faces = inconsistent
            for &ci in faces {
                consistent[ci] = 0.0;
            }
        }
        // Check reverse: for manifold meshes, (b,a) should exist once
        if let Some(rev) = directed.get(&(b, a)) {
            if rev.len() > 1 {
                for &ci in rev {
                    consistent[ci] = 0.0;
                }
            }
        }
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Consistent",
            consistent,
            1,
        )));
    result
}

/// Flip the winding of all faces (reverse vertex order).
pub fn flip_all_faces(mesh: &PolyData) -> PolyData {
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let reversed: Vec<i64> = cell.iter().rev().cloned().collect();
        polys.push_cell(&reversed);
    }
    let mut result = mesh.clone();
    result.polys = polys;
    result
}

/// Flip only inconsistent faces to match the majority orientation.
pub fn repair_face_orientation(mesh: &PolyData) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();
    if n_cells == 0 {
        return mesh.clone();
    }

    let mut edge_adj: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_adj.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    // BFS to orient consistently from seed face
    let mut oriented = vec![false; n_cells];
    let mut flipped = vec![false; n_cells];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(0);
    oriented[0] = true;

    while let Some(ci) = queue.pop_front() {
        let cell = &all_cells[ci];
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let edge = (a.min(b), a.max(b));
            if let Some(nbs) = edge_adj.get(&edge) {
                for &ni in nbs {
                    if oriented[ni] {
                        continue;
                    }
                    oriented[ni] = true;
                    // Check if neighbor shares this edge in same direction (needs flip)
                    let nb_cell = &all_cells[ni];
                    let nnc = nb_cell.len();
                    let same_dir = (0..nnc).any(|j| {
                        let na = nb_cell[j] as usize;
                        let nb = nb_cell[(j + 1) % nnc] as usize;
                        (na == a && nb == b) || (na == b && nb == a && flipped[ci])
                    });
                    // If both use (a,b) in same direction, one needs flipping
                    let needs_flip = (0..nnc).any(|j| {
                        let na = nb_cell[j] as usize;
                        let nb_v = nb_cell[(j + 1) % nnc] as usize;
                        if flipped[ci] {
                            na == b && nb_v == a
                        } else {
                            na == a && nb_v == b
                        }
                    });
                    if needs_flip {
                        flipped[ni] = true;
                    }
                    queue.push_back(ni);
                }
            }
        }
    }

    let mut polys = CellArray::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        if flipped[ci] {
            let reversed: Vec<i64> = cell.iter().rev().cloned().collect();
            polys.push_cell(&reversed);
        } else {
            polys.push_cell(cell);
        }
    }

    let mut result = mesh.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_consistent() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        ); // consistently oriented
        let result = check_face_orientation(&mesh);
        assert!(result.cell_data().get_array("Consistent").is_some());
    }
    #[test]
    fn flip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let flipped = flip_all_faces(&mesh);
        // First vertex should now be last
        let cell: Vec<i64> = flipped.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell[0], 2);
    }
    #[test]
    fn repair() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [2, 3, 1]],
        ); // second face has wrong winding
        let repaired = repair_face_orientation(&mesh);
        assert_eq!(repaired.polys.num_cells(), 2);
    }
}
