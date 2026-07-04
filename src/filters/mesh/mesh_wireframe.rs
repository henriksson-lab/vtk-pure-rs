//! Extract wireframe (all edges as lines) from a mesh.
use crate::data::{CellArray, PolyData};
use std::collections::BTreeSet;

pub fn wireframe(mesh: &PolyData) -> PolyData {
    let mut edges = BTreeSet::new();
    let mut lines = CellArray::new();
    let n = mesh.points.len();
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(edge[0], edge[1], n, &mut edges);
        }
    }
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            add_edge(cell[i], cell[(i + 1) % nc], n, &mut edges);
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            add_edge(tri[0], tri[1], n, &mut edges);
            add_edge(tri[1], tri[2], n, &mut edges);
            add_edge(tri[2], tri[0], n, &mut edges);
        }
    }
    for (a, b) in edges {
        lines.push_cell(&[a, b]);
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.lines = lines;
    result
}

fn add_edge(a: i64, b: i64, n: usize, edges: &mut BTreeSet<(i64, i64)>) {
    let (Some(a), Some(b)) = (valid_point_index(a, n), valid_point_index(b, n)) else {
        return;
    };
    if a == b {
        return;
    }
    edges.insert((a.min(b), a.max(b)));
}

fn valid_point_index(id: i64, n: usize) -> Option<i64> {
    usize::try_from(id)
        .ok()
        .filter(|&id| id < n)
        .map(|id| id as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wireframe() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = wireframe(&mesh);
        assert_eq!(r.lines.num_cells(), 3);
    }
}
