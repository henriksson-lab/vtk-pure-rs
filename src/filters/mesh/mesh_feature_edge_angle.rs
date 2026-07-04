//! Compute dihedral angle at each edge and classify.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_edges_by_angle_range(mesh: &PolyData, min_angle: f64, max_angle: f64) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let nc = c.len();
        for i in 0..nc {
            let a = c[i];
            let b = c[(i + 1) % nc];
            if a >= 0 && b >= 0 {
                let a = a as usize;
                let b = b as usize;
                if a < mesh.points.len() && b < mesh.points.len() {
                    ef.entry((a.min(b), a.max(b))).or_default().push(ci);
                }
            }
        }
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), faces) in &ef {
        if faces.len() != 2 {
            continue;
        }
        let n0 = polygon_normal(&cells[faces[0]], mesh);
        let n1 = polygon_normal(&cells[faces[1]], mesh);
        let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
        let angle = dot.acos().to_degrees();
        if angle >= min_angle && angle <= max_angle {
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

pub fn extract_crease_edges(mesh: &PolyData, threshold: f64) -> PolyData {
    extract_edges_by_angle_range(mesh, threshold, 180.0)
}
pub fn extract_smooth_edges(mesh: &PolyData, threshold: f64) -> PolyData {
    extract_edges_by_angle_range(mesh, 0.0, threshold)
}

fn polygon_normal(c: &[i64], m: &PolyData) -> [f64; 3] {
    if c.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    if c.iter().any(|&v| v < 0 || v as usize >= m.points.len()) {
        return [0.0, 0.0, 1.0];
    }
    let mut n = [0.0f64; 3];
    for i in 0..c.len() {
        let p = m.points.get(c[i] as usize);
        let q = m.points.get(c[(i + 1) % c.len()] as usize);
        n[0] += (p[1] - q[1]) * (p[2] + q[2]);
        n[1] += (p[2] - q[2]) * (p[0] + q[0]);
        n[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / l, n[1] / l, n[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_crease() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = extract_crease_edges(&m, 20.0);
        assert!(r.lines.num_cells() >= 1);
    }
    #[test]
    fn test_smooth() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = extract_smooth_edges(&m, 10.0);
        assert!(r.lines.num_cells() >= 1);
    }
}
