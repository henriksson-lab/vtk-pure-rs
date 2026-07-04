//! Extract boundary loops from a mesh as polylines.

use crate::data::{CellArray, Points, PolyData};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Extract boundary edges as polyline loops.
pub fn extract_boundary_loops(mesh: &PolyData) -> PolyData {
    // Find boundary edges (shared by exactly 1 face)
    let edge_count = polygon_edge_counts(mesh);
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
    let mut adj: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for &(a, b) in &boundary_edges {
        adj.entry(a).or_default().insert(b);
        adj.entry(b).or_default().insert(a);
    }

    // Trace loops
    let mut visited_edges: BTreeSet<(usize, usize)> = BTreeSet::new();
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
        let mut closed = false;
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
                        loop_verts.push(nb);
                        closed = true;
                        break;
                    }
                    loop_verts.push(nb);
                    current = nb;
                }
                None => break,
            }
        }
        if closed && loop_verts.len() >= 4 {
            loops.push(loop_verts);
        }
    }

    // Build output
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pt_map: HashMap<usize, usize> = HashMap::new();

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

fn polygon_edge_counts(mesh: &PolyData) -> BTreeMap<(usize, usize), usize> {
    let mut edge_count = BTreeMap::new();
    let n_points = mesh.points.len();

    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_counted_edge(
                &mut edge_count,
                n_points,
                cell[i],
                cell[(i + 1) % cell.len()],
            );
        }
    }

    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_counted_edge(&mut edge_count, n_points, tri[0], tri[1]);
            insert_counted_edge(&mut edge_count, n_points, tri[1], tri[2]);
            insert_counted_edge(&mut edge_count, n_points, tri[2], tri[0]);
        }
    }

    edge_count
}

fn insert_counted_edge(
    edge_count: &mut BTreeMap<(usize, usize), usize>,
    n_points: usize,
    a: i64,
    b: i64,
) {
    let (Some(a), Some(b)) = (
        valid_point_index(a, n_points),
        valid_point_index(b, n_points),
    ) else {
        return;
    };
    if a != b {
        *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
    }
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
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
        let loops = extract_boundary_loops(&mesh);
        let line = loops.lines.cell(0);
        assert_eq!(line.len(), 4);
        assert_eq!(line.first(), line.last());
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
