//! Extract sharp feature lines from a mesh based on dihedral angle.
use crate::data::{CellArray, Points, PolyData};

pub fn feature_lines(mesh: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.is_empty() {
        return PolyData::new();
    }
    let threshold = angle_threshold_deg * std::f64::consts::PI / 180.0;
    // Compute face normals
    let face_normals: Vec<[f64; 3]> = tris
        .iter()
        .map(|&[a, b, c]| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let nx = u[1] * v[2] - u[2] * v[1];
            let ny = u[2] * v[0] - u[0] * v[2];
            let nz = u[0] * v[1] - u[1] * v[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len > 1e-15 {
                [nx / len, ny / len, nz / len]
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect();
    // Build edge-to-face map
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, &[a, b, c]) in tris.iter().enumerate() {
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
            edge_faces.entry(e).or_default().push(fi);
        }
    }
    let mut lines = CellArray::new();
    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = face_normals[faces[0]];
            let n1 = face_normals[faces[1]];
            let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
            let angle = dot.clamp(-1.0, 1.0).acos();
            if angle > threshold {
                lines.push_cell(&[a as i64, b as i64]);
            }
        } else if faces.len() == 1 {
            // Boundary edge
            lines.push_cell(&[a as i64, b as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.lines = lines;
    result
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
