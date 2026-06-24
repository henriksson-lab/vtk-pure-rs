use std::collections::HashMap;

use crate::data::{CellArray, PolyData};

/// Orient all polygons in a PolyData to have consistent winding order.
///
/// Uses a breadth-first traversal starting from the first polygon.
/// Neighboring polygons sharing an edge are flipped if their shared edge
/// has the same direction (indicating inconsistent winding). Only works
/// on manifold meshes (each edge shared by at most two polygons).
pub fn orient(input: &PolyData) -> PolyData {
    let nc = input.polys.num_cells();
    if nc == 0 {
        return input.clone();
    }

    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();

    // Build edge -> cell adjacency using packed u64 keys.
    // Store (cell_idx, is_forward) packed into u64: high 32 = cell, bit 0 = forward.
    let mut edge_adj: HashMap<u64, [u64; 2]> = HashMap::with_capacity(nc * 2);

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let n = end - start;
        for i in 0..n {
            let a = conn[start + i];
            let b = conn[start + if i + 1 < n { i + 1 } else { 0 }];
            let key = if a < b {
                (a as u64) << 32 | b as u64
            } else {
                (b as u64) << 32 | a as u64
            };
            let forward = a < b;
            let val = ((ci as u64) << 1) | (forward as u64);
            let entry = edge_adj.entry(key).or_insert([u64::MAX, u64::MAX]);
            if entry[0] == u64::MAX {
                entry[0] = val;
            } else if entry[1] == u64::MAX {
                entry[1] = val;
            }
        }
    }

    // BFS to determine which cells need flipping
    // 0 = unvisited, 1 = keep, 2 = flip
    let mut state = vec![0u8; nc];
    let mut queue = std::collections::VecDeque::with_capacity(nc);

    for start_ci in 0..nc {
        if state[start_ci] != 0 {
            continue;
        }
        state[start_ci] = 1; // keep
        queue.push_back(start_ci);

        while let Some(ci) = queue.pop_front() {
            let ci_flipped = state[ci] == 2;
            let s = offsets[ci] as usize;
            let e = offsets[ci + 1] as usize;
            let n = e - s;

            for i in 0..n {
                let a = conn[s + i];
                let b = conn[s + if i + 1 < n { i + 1 } else { 0 }];
                let key = if a < b {
                    (a as u64) << 32 | b as u64
                } else {
                    (b as u64) << 32 | a as u64
                };
                let ci_forward = a < b;
                let ci_actual_forward = ci_forward ^ ci_flipped;

                if let Some(pair) = edge_adj.get(&key) {
                    for &packed in pair {
                        if packed == u64::MAX {
                            break;
                        }
                        let ni = (packed >> 1) as usize;
                        let ni_forward = (packed & 1) != 0;
                        if ni == ci || state[ni] != 0 {
                            continue;
                        }
                        let should_flip = ci_actual_forward == ni_forward;
                        state[ni] = if should_flip { 2 } else { 1 };
                        queue.push_back(ni);
                    }
                }
            }
        }
    }

    // Build output using raw connectivity — only reverse flipped cells
    let mut out_off = Vec::with_capacity(nc + 1);
    let mut out_conn = Vec::with_capacity(conn.len());
    out_off.push(0i64);

    for ci in 0..nc {
        let s = offsets[ci] as usize;
        let e = offsets[ci + 1] as usize;
        if state[ci] == 2 {
            // Reverse the cell
            for idx in (s..e).rev() {
                out_conn.push(conn[idx]);
            }
        } else {
            out_conn.extend_from_slice(&conn[s..e]);
        }
        out_off.push(out_conn.len() as i64);
    }

    let mut pd = input.clone();
    pd.polys = CellArray::from_raw(out_off, out_conn);
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_consistent() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            // Both CCW when viewed from +Z
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = orient(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        // Check that shared edge (1,2) is traversed in opposite directions
        let c0 = result.polys.cell(0);
        let c1 = result.polys.cell(1);
        // In c0: 1->2, in c1: should be 2->1 (opposite)
        let edge_forward_in_c0 = has_directed_edge(c0, 1, 2);
        let edge_forward_in_c1 = has_directed_edge(c1, 1, 2);
        assert_ne!(edge_forward_in_c0, edge_forward_in_c1);
    }

    #[test]
    fn fixes_inconsistent_winding() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            // Second triangle has same winding direction for shared edge (inconsistent)
            vec![[0, 1, 2], [1, 2, 3]],
        );
        let result = orient(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        // After orientation, shared edge should go in opposite directions
        let c0 = result.polys.cell(0);
        let c1 = result.polys.cell(1);
        let edge_forward_in_c0 = has_directed_edge(c0, 1, 2);
        let edge_forward_in_c1 = has_directed_edge(c1, 1, 2);
        assert_ne!(edge_forward_in_c0, edge_forward_in_c1);
    }

    fn has_directed_edge(cell: &[i64], a: i64, b: i64) -> bool {
        let n = cell.len();
        for i in 0..n {
            if cell[i] == a && cell[(i + 1) % n] == b {
                return true;
            }
        }
        false
    }
}
