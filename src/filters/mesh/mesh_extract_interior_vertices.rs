//! Extract interior (non-boundary) vertices.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_interior_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..n {
        if !boundary.contains(&i) {
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
pub fn count_interior_vertices(mesh: &PolyData) -> usize {
    let n = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    n - boundary.len()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 3], [1, 4, 3], [0, 3, 2], [3, 4, 2]],
        );
        let c = count_interior_vertices(&m);
        assert!(c >= 1);
    }
}
