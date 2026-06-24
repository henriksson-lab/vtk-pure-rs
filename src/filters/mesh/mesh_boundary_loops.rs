//! Extract boundary loops (ordered closed polylines) from a mesh.
use crate::data::{CellArray, PolyData};

pub fn boundary_loops(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    // Find boundary edges (shared by exactly one face)
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    let mut boundary_adj: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }
    let mut lines = CellArray::new();
    let mut visited_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    for &start in boundary_adj.keys() {
        if boundary_adj.get(&start).map_or(true, |v| {
            v.iter().all(|&nb| {
                let e = if start < nb { (start, nb) } else { (nb, start) };
                visited_edges.contains(&e)
            })
        }) {
            continue;
        }
        // Trace loop
        let mut current = start;
        let mut loop_verts = vec![start];
        loop {
            let neighbors = boundary_adj.get(&current).unwrap();
            let next = neighbors.iter().find(|&&nb| {
                let e = if current < nb {
                    (current, nb)
                } else {
                    (nb, current)
                };
                !visited_edges.contains(&e)
            });
            match next {
                Some(&nb) => {
                    let e = if current < nb {
                        (current, nb)
                    } else {
                        (nb, current)
                    };
                    visited_edges.insert(e);
                    if nb == start {
                        break;
                    }
                    loop_verts.push(nb);
                    current = nb;
                }
                None => break,
            }
        }
        if loop_verts.len() > 1 {
            for i in 0..loop_verts.len() {
                let j = (i + 1) % loop_verts.len();
                lines.push_cell(&[loop_verts[i] as i64, loop_verts[j] as i64]);
            }
        }
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.lines = lines;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boundary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boundary_loops(&mesh);
        assert_eq!(r.lines.num_cells(), 3); // triangle has 3 boundary edges forming 1 loop
    }
}
