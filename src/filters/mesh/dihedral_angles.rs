//! Compute dihedral angles between adjacent faces.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Result of dihedral angle analysis.
pub struct DihedralAngleStats {
    pub min_degrees: f64,
    pub max_degrees: f64,
    pub mean_degrees: f64,
    pub num_edges: usize,
}

/// Compute dihedral angles at all interior edges and attach as cell data.
pub fn dihedral_angle_analysis(mesh: &PolyData) -> DihedralAngleStats {
    let angles = compute_dihedral_angles(mesh);
    if angles.is_empty() {
        return DihedralAngleStats {
            min_degrees: 0.0,
            max_degrees: 0.0,
            mean_degrees: 0.0,
            num_edges: 0,
        };
    }
    let mn = angles.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = angles.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = angles.iter().sum::<f64>() / angles.len() as f64;
    DihedralAngleStats {
        min_degrees: mn,
        max_degrees: mx,
        mean_degrees: mean,
        num_edges: angles.len(),
    }
}

/// Compute dihedral angle (in degrees) for each interior edge.
pub fn compute_dihedral_angles(mesh: &PolyData) -> Vec<f64> {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut angles = Vec::new();
    for (_, faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = face_normal(&cells[faces[0]], mesh);
            let n1 = face_normal(&cells[faces[1]], mesh);
            let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
            angles.push(dot.acos().to_degrees());
        }
    }
    angles
}

/// Attach per-edge dihedral angle as a point data array (averaged at vertices).
pub fn attach_dihedral_angles(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut vert_angles = vec![Vec::new(); n];
    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = face_normal(&cells[faces[0]], mesh);
            let n1 = face_normal(&cells[faces[1]], mesh);
            let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
            let angle = dot.acos().to_degrees();
            vert_angles[a].push(angle);
            vert_angles[b].push(angle);
        }
    }

    let data: Vec<f64> = vert_angles
        .iter()
        .map(|va| {
            if va.is_empty() {
                0.0
            } else {
                va.iter().sum::<f64>() / va.len() as f64
            }
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DihedralAngle",
            data,
            1,
        )));
    result
}

fn face_normal(cell: &[i64], mesh: &PolyData) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let stats = dihedral_angle_analysis(&mesh);
        assert_eq!(stats.num_edges, 1);
        assert!(stats.min_degrees < 1.0); // coplanar -> ~0 degrees
    }
    #[test]
    fn test_right_angle() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 0.0, 1.0],
                [0.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let stats = dihedral_angle_analysis(&mesh);
        assert!(stats.num_edges >= 1);
    }
    #[test]
    fn test_attach() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = attach_dihedral_angles(&mesh);
        assert!(r.point_data().get_array("DihedralAngle").is_some());
    }
}
