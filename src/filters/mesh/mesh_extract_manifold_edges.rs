//! Extract manifold, non-manifold, and boundary edges separately.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_manifold_edges(mesh: &PolyData) -> PolyData {
    extract_edges_by_count(mesh, 2)
}
pub fn extract_non_manifold_edges(mesh: &PolyData) -> PolyData {
    extract_edges_by_count_above(mesh, 2)
}
pub fn extract_boundary_edges_only(mesh: &PolyData) -> PolyData {
    extract_edges_by_count(mesh, 1)
}
fn extract_edges_by_count(mesh: &PolyData, target: usize) -> PolyData {
    let ec = edge_counts(mesh);
    build_edge_mesh(mesh, &ec, |c| c == target)
}
fn extract_edges_by_count_above(mesh: &PolyData, target: usize) -> PolyData {
    let ec = edge_counts(mesh);
    build_edge_mesh(mesh, &ec, |c| c > target)
}
fn edge_counts(mesh: &PolyData) -> std::collections::HashMap<(usize, usize), usize> {
    let mut ec = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    ec
}
fn build_edge_mesh(
    mesh: &PolyData,
    ec: &std::collections::HashMap<(usize, usize), usize>,
    pred: impl Fn(usize) -> bool,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), &c) in ec {
        if pred(c) {
            let ia = *pm.entry(a).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(a));
                i
            });
            let ib = *pm.entry(b).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(b));
                i
            });
            lines.push_cell(&[ia as i64, ib as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn edge_type_counts(mesh: &PolyData) -> (usize, usize, usize) {
    let ec = edge_counts(mesh);
    let boundary = ec.values().filter(|&&c| c == 1).count();
    let manifold = ec.values().filter(|&&c| c == 2).count();
    let non_manifold = ec.values().filter(|&&c| c > 2).count();
    (boundary, manifold, non_manifold)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boundary() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extract_boundary_edges_only(&m);
        assert_eq!(r.lines.num_cells(), 3);
    }
    #[test]
    fn test_manifold() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = extract_manifold_edges(&m);
        assert_eq!(r.lines.num_cells(), 1);
    }
    #[test]
    fn test_counts() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let (b, man, nm) = edge_type_counts(&m);
        assert_eq!(b, 4);
        assert_eq!(man, 1);
        assert_eq!(nm, 0);
    }
}
