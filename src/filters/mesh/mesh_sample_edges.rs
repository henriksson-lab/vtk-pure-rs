//! Sample points along mesh edges.

use crate::data::{CellArray, Points, PolyData};

/// Sample N equidistant points along each edge.
pub fn sample_edges(mesh: &PolyData, samples_per_edge: usize) -> PolyData {
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let s = samples_per_edge.max(1);
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let key = (a.min(b), a.max(b));
            if !seen.insert(key) {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            for j in 0..=s {
                let t = j as f64 / s as f64;
                let idx = pts.len();
                pts.push([
                    pa[0] + (pb[0] - pa[0]) * t,
                    pa[1] + (pb[1] - pa[1]) * t,
                    pa[2] + (pb[2] - pa[2]) * t,
                ]);
                verts.push_cell(&[idx as i64]);
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}

/// Sample points at edge midpoints only.
pub fn edge_midpoints(mesh: &PolyData) -> PolyData {
    sample_edges(mesh, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sample() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = sample_edges(&mesh, 3);
        assert_eq!(r.points.len(), 12); // 3 edges * 4 points each
    }
    #[test]
    fn test_midpoints() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = edge_midpoints(&mesh);
        assert_eq!(r.points.len(), 6); // 3 edges * 2 points each
    }
}
