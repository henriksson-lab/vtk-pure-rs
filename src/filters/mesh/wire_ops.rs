//! Wireframe and edge extraction operations.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract wireframe (all edges as lines).
pub fn extract_wireframe(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if seen.insert((a.min(b), a.max(b))) {
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
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}

/// Extract only boundary edges as lines.
pub fn extract_boundary_wireframe(mesh: &PolyData) -> PolyData {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), &count) in &ec {
        if count == 1 {
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

/// Extract internal edges only (shared by 2 faces).
pub fn extract_internal_edges(mesh: &PolyData) -> PolyData {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), &count) in &ec {
        if count == 2 {
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

/// Count edges by type.
pub fn edge_counts(mesh: &PolyData) -> (usize, usize, usize) {
    // (boundary, internal, non-manifold)
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let boundary = ec.values().filter(|&&c| c == 1).count();
    let internal = ec.values().filter(|&&c| c == 2).count();
    let non_manifold = ec.values().filter(|&&c| c > 2).count();
    (boundary, internal, non_manifold)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wireframe() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let wf = extract_wireframe(&mesh);
        assert_eq!(wf.lines.num_cells(), 3);
    }
    #[test]
    fn boundary() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let bnd = extract_boundary_wireframe(&mesh);
        assert_eq!(bnd.lines.num_cells(), 4); // 4 boundary edges
    }
    #[test]
    fn internal() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let int = extract_internal_edges(&mesh);
        assert_eq!(int.lines.num_cells(), 1); // edge 1-2 is shared
    }
    #[test]
    fn counts() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let (b, i, nm) = edge_counts(&mesh);
        assert_eq!(b, 4);
        assert_eq!(i, 1);
        assert_eq!(nm, 0);
    }
}
