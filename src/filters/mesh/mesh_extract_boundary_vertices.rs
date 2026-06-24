//! Extract boundary vertices as a point cloud.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_boundary_vertices(mesh: &PolyData) -> PolyData {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut bv: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            bv.insert(a);
            bv.insert(b);
        }
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for &v in &bv {
        let idx = pts.len();
        pts.push(mesh.points.get(v));
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}
pub fn is_boundary_vertex(mesh: &PolyData, vertex: usize) -> bool {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    ec.iter()
        .any(|(&(a, b), &c)| c == 1 && (a == vertex || b == vertex))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extract_boundary_vertices(&m);
        assert_eq!(r.points.len(), 3);
    }
    #[test]
    fn test_is_boundary() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert!(is_boundary_vertex(&m, 0));
    }
}
