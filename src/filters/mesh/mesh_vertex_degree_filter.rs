//! Filter mesh by vertex degree (valence).
use crate::data::{CellArray, Points, PolyData};
pub fn extract_vertices_by_degree(mesh: &PolyData, min_deg: usize, max_deg: usize) -> PolyData {
    let n = mesh.points.len();
    let mut deg = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                deg[a].insert(b);
                deg[b].insert(a);
            }
        }
    }
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
    let mut deg = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                deg[a].insert(b);
                deg[b].insert(a);
            }
        }
    }
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
