//! Edge collapse mesh simplification (greedy shortest-edge first).
use crate::data::{CellArray, Points, PolyData};

pub fn edge_collapse(mesh: &PolyData, target_faces: usize) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<usize>> = mesh
        .polys
        .iter()
        .filter_map(|c| valid_point_ids(c, n))
        .collect();
    let n_cells = cells.len();
    if n_cells <= target_faces {
        return mesh.clone();
    }

    // Union-Find
    let mut parent: Vec<usize> = (0..n).collect();

    // Collect edges with lengths
    let mut edges: Vec<(f64, usize, usize)> = Vec::new();
    for cell in &cells {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            let (a, b) = (a.min(b), a.max(b));
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            edges.push((d, a, b));
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
            let mapped = compact_mapped_cell(cell, &mut parent);
            if mapped.len() < 3 {
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
        let compact = compact_mapped_cell(cell, &mut parent);
        if compact.len() < 3 {
            continue;
        }
        let mapped: Vec<i64> = compact.iter().map(|&v| new_idx[v] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

fn valid_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    let ids: Option<Vec<usize>> = cell
        .iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
        .collect();
    ids.filter(|ids| ids.len() >= 3)
}

fn find(p: &mut Vec<usize>, x: usize) -> usize {
    if p[x] != x {
        p[x] = find(p, p[x]);
    }
    p[x]
}

fn compact_mapped_cell(cell: &[usize], parent: &mut Vec<usize>) -> Vec<usize> {
    let mut mapped = Vec::with_capacity(cell.len());
    for &v in cell {
        let r = find(parent, v);
        if mapped.last().copied() != Some(r) {
            mapped.push(r);
        }
    }
    if mapped.len() > 1 && mapped.first() == mapped.last() {
        mapped.pop();
    }
    let unique: std::collections::HashSet<usize> = mapped.iter().copied().collect();
    if unique.len() != mapped.len() {
        return Vec::new();
    }
    mapped
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
