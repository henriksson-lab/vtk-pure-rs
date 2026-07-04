//! Extract sharp feature lines from a mesh based on dihedral angle.
use crate::data::{CellArray, Points, PolyData};

pub fn feature_lines(mesh: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let cells: Vec<Vec<usize>> = mesh
        .polys
        .iter()
        .filter_map(|cell| {
            let converted: Option<Vec<usize>> = cell
                .iter()
                .map(|&v| {
                    let vi = v as usize;
                    (vi < mesh.points.len()).then_some(vi)
                })
                .collect();
            converted.filter(|c| c.len() >= 2)
        })
        .collect();
    if cells.is_empty() {
        return PolyData::new();
    }
    let threshold = angle_threshold_deg * std::f64::consts::PI / 180.0;
    // Compute face normals
    let face_normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|cell| polygon_normal(cell, mesh))
        .collect();
    // Build edge-to-face map
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let e0 = cell[i];
            let e1 = cell[(i + 1) % nc];
            if e0 == e1 {
                continue;
            }
            let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
            edge_faces.entry(e).or_default().push(fi);
        }
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), faces) in &edge_faces {
        let is_feature = if faces.len() == 2 {
            let n0 = face_normals[faces[0]];
            let n1 = face_normals[faces[1]];
            let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
            let angle = dot.clamp(-1.0, 1.0).acos();
            angle >= threshold
        } else if faces.len() == 1 {
            true
        } else {
            true
        };
        if !is_feature {
            continue;
        }
        let ia = *pt_map.entry(a).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(a));
            i
        });
        let ib = *pt_map.entry(b).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(b));
            i
        });
        lines.push_cell(&[ia as i64, ib as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

fn polygon_normal(cell: &[usize], mesh: &PolyData) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let mut n = [0.0f64; 3];
    for i in 0..cell.len() {
        let p = mesh.points.get(cell[i]);
        let q = mesh.points.get(cell[(i + 1) % cell.len()]);
        n[0] += (p[1] - q[1]) * (p[2] + q[2]);
        n[1] += (p[2] - q[2]) * (p[0] + q[0]);
        n[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_feature_lines() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let r = feature_lines(&mesh, 30.0);
        assert!(r.lines.num_cells() > 0);
    }
}
