//! Edge collapse mesh simplification (greedy shortest-edge first).
use crate::data::{CellArray, Points, PolyData};

pub fn edge_collapse(mesh: &PolyData, target_faces: usize) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = cells.len();
    if n_cells <= target_faces {
        return mesh.clone();
    }

    // Union-Find
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut Vec<usize>, x: usize) -> usize {
        if p[x] != x {
            p[x] = find(p, p[x]);
        }
        p[x]
    }

    // Collect edges with lengths
    let mut edges: Vec<(f64, usize, usize)> = Vec::new();
    for cell in &cells {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < b && a < n && b < n {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                edges.push((d, a, b));
            }
        }
    }
    edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut active_cells = vec![true; n_cells];
    let mut remaining = n_cells;

    for &(_, a, b) in &edges {
        if remaining <= target_faces {
            break;
        }
        let ra = find(&mut parent, a);
        let rb = find(&mut parent, b);
        if ra == rb {
            continue;
        }
        parent[rb] = ra;
        // Deactivate degenerate cells
        for (ci, cell) in cells.iter().enumerate() {
            if !active_cells[ci] {
                continue;
            }
            let mapped: Vec<usize> = cell
                .iter()
                .map(|&v| find(&mut parent, v as usize))
                .collect();
            let unique: std::collections::HashSet<usize> = mapped.iter().copied().collect();
            if unique.len() < 3 {
                active_cells[ci] = false;
                remaining -= 1;
            }
        }
    }

    // Rebuild with collapsed vertices
    let mut new_idx = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    let mut seen = std::collections::HashMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        if let Some(&idx) = seen.get(&r) {
            new_idx[i] = idx;
        } else {
            let idx = pts.len();
            pts.push(mesh.points.get(r));
            new_idx[i] = idx;
            seen.insert(r, idx);
        }
    }
    let mut polys = CellArray::new();
    for (ci, cell) in cells.iter().enumerate() {
        if !active_cells[ci] {
            continue;
        }
        let mapped: Vec<i64> = cell.iter().map(|&v| new_idx[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_edge_collapse() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let r = edge_collapse(&mesh, 1);
        assert!(r.polys.num_cells() <= 3);
    }
}
