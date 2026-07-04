//! Detect sharp edges and feature lines on meshes.

use crate::data::{CellArray, Points, PolyData};

/// Extract edges with dihedral angle above threshold (in degrees).
pub fn extract_sharp_edges(mesh: &PolyData, angle_threshold: f64) -> PolyData {
    let cos_thresh = angle_threshold.to_radians().cos();
    let cells: Vec<Vec<usize>> = mesh
        .polys
        .iter()
        .filter_map(|c| valid_point_ids(c, mesh.points.len()))
        .collect();
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let n0 = face_normal(&cells[faces[0]], mesh);
        let n1 = face_normal(&cells[faces[1]], mesh);
        let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
        if dot <= cos_thresh {
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

fn face_normal(cell: &[usize], mesh: &PolyData) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;
    for i in 0..cell.len() {
        let p = mesh.points.get(cell[i]);
        let q = mesh.points.get(cell[(i + 1) % cell.len()]);
        nx += (p[1] - q[1]) * (p[2] + q[2]);
        ny += (p[2] - q[2]) * (p[0] + q[0]);
        nz += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let l = (nx * nx + ny * ny + nz * nz).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [nx / l, ny / l, nz / l]
    }
}

fn valid_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    let ids: Option<Vec<usize>> = cell
        .iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
        .collect();
    ids.filter(|ids| ids.len() >= 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sharp() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = extract_sharp_edges(&mesh, 30.0);
        assert!(r.lines.num_cells() >= 1);
    }
    #[test]
    fn test_flat() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = extract_sharp_edges(&mesh, 10.0);
        assert_eq!(r.lines.num_cells(), 0); // coplanar
    }
}
