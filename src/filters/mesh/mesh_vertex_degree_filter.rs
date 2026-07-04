//! Filter mesh by vertex degree (valence).
use crate::data::{CellArray, Points, PolyData};
use std::collections::HashSet;

pub fn extract_vertices_by_degree(mesh: &PolyData, min_deg: usize, max_deg: usize) -> PolyData {
    let n = mesh.points.len();
    let deg = build_vertex_degrees(mesh);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..n {
        let d = deg[i].len();
        if d >= min_deg && d <= max_deg {
            let idx = pts.len();
            pts.push(mesh.points.get(i));
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}
pub fn extract_irregular_vertices(mesh: &PolyData) -> PolyData {
    // Regular interior vertex in triangulation has degree 6
    extract_vertices_excluding_degree(mesh, 6)
}
fn extract_vertices_excluding_degree(mesh: &PolyData, exclude: usize) -> PolyData {
    let n = mesh.points.len();
    let deg = build_vertex_degrees(mesh);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..n {
        if deg[i].len() != exclude && deg[i].len() > 0 {
            let idx = pts.len();
            pts.push(mesh.points.get(i));
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}

fn build_vertex_degrees(mesh: &PolyData) -> Vec<HashSet<usize>> {
    let n = mesh.points.len();
    let mut deg = vec![HashSet::new(); n];

    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            insert_edge(&mut deg, n, edge[0], edge[1]);
        }
    }

    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(&mut deg, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }

    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut deg, n, tri[0], tri[1]);
            insert_edge(&mut deg, n, tri[1], tri[2]);
            insert_edge(&mut deg, n, tri[2], tri[0]);
        }
    }

    deg
}

fn insert_edge(deg: &mut [HashSet<usize>], n: usize, a: i64, b: i64) {
    let (Some(a), Some(b)) = (valid_point_index(a, n), valid_point_index(b, n)) else {
        return;
    };
    if a == b {
        return;
    }
    deg[a].insert(b);
    deg[b].insert(a);
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_degree() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = extract_vertices_by_degree(&m, 3, 3);
        assert!(r.points.len() >= 2);
    } // vertices 1,2 have degree 3
    #[test]
    fn test_irregular() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extract_irregular_vertices(&m);
        assert!(r.points.len() >= 1);
    } // all have deg 2, != 6
}
