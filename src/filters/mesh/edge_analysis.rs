//! Mesh edge analysis: edge statistics, sharp/smooth classification.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Edge analysis results.
#[derive(Debug, Clone)]
pub struct EdgeAnalysis {
    pub total_edges: usize,
    pub boundary_edges: usize,
    pub internal_edges: usize,
    pub non_manifold_edges: usize,
    pub sharp_edges: usize,
    pub min_length: f64,
    pub max_length: f64,
    pub mean_length: f64,
    pub total_length: f64,
    pub min_dihedral: f64,
    pub max_dihedral: f64,
    pub mean_dihedral: f64,
}

impl std::fmt::Display for EdgeAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Edges: {} (boundary={}, internal={}, non-manifold={}, sharp={}), \
               length=[{:.4},{:.4}] mean={:.4}, dihedral=[{:.1}°,{:.1}°] mean={:.1}°",
            self.total_edges,
            self.boundary_edges,
            self.internal_edges,
            self.non_manifold_edges,
            self.sharp_edges,
            self.min_length,
            self.max_length,
            self.mean_length,
            self.min_dihedral.to_degrees(),
            self.max_dihedral.to_degrees(),
            self.mean_dihedral.to_degrees()
        )
    }
}

/// Compute comprehensive edge analysis.
pub fn analyze_edges(mesh: &PolyData, sharp_angle_degrees: f64) -> EdgeAnalysis {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let face_normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    let mut edge_map: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_map.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let sharp_cos = (sharp_angle_degrees * std::f64::consts::PI / 180.0).cos();
    let mut boundary = 0;
    let mut internal = 0;
    let mut non_manifold = 0;
    let mut sharp = 0;
    let mut lengths = Vec::new();
    let mut dihedrals = Vec::new();

    for (&(a, b), faces) in &edge_map {
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let len =
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        lengths.push(len);

        match faces.len() {
            1 => boundary += 1,
            2 => {
                internal += 1;
                let dot = face_normals[faces[0]][0] * face_normals[faces[1]][0]
                    + face_normals[faces[0]][1] * face_normals[faces[1]][1]
                    + face_normals[faces[0]][2] * face_normals[faces[1]][2];
                let angle = dot.clamp(-1.0, 1.0).acos();
                dihedrals.push(angle);
                if dot < sharp_cos {
                    sharp += 1;
                }
            }
            _ => non_manifold += 1,
        }
    }

    let total = edge_map.len();
    let total_len: f64 = lengths.iter().sum();
    let total_dih: f64 = dihedrals.iter().sum();

    EdgeAnalysis {
        total_edges: total,
        boundary_edges: boundary,
        internal_edges: internal,
        non_manifold_edges: non_manifold,
        sharp_edges: sharp,
        min_length: lengths.iter().cloned().fold(f64::MAX, f64::min),
        max_length: lengths.iter().cloned().fold(0.0f64, f64::max),
        mean_length: if total > 0 {
            total_len / total as f64
        } else {
            0.0
        },
        total_length: total_len,
        min_dihedral: dihedrals.iter().cloned().fold(f64::MAX, f64::min),
        max_dihedral: dihedrals.iter().cloned().fold(0.0f64, f64::max),
        mean_dihedral: if !dihedrals.is_empty() {
            total_dih / dihedrals.len() as f64
        } else {
            0.0
        },
    }
}

/// Extract sharp edges as a line PolyData.
pub fn extract_sharp_edges_with_angle(mesh: &PolyData, sharp_angle_degrees: f64) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();
    let sharp_cos = (sharp_angle_degrees * std::f64::consts::PI / 180.0).cos();

    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut angle_data = Vec::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let dot = normals[faces[0]][0] * normals[faces[1]][0]
            + normals[faces[0]][1] * normals[faces[1]][1]
            + normals[faces[0]][2] * normals[faces[1]][2];
        if dot >= sharp_cos {
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
        angle_data.push(dot.clamp(-1.0, 1.0).acos().to_degrees());
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DihedralAngle",
            angle_data,
            1,
        )));
    result
}

fn face_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
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
    fn single_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let analysis = analyze_edges(&mesh, 30.0);
        assert_eq!(analysis.total_edges, 3);
        assert_eq!(analysis.boundary_edges, 3);
        assert_eq!(analysis.internal_edges, 0);
    }

    #[test]
    fn cube_sharp_edges() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        );
        let analysis = analyze_edges(&mesh, 30.0);
        assert!(analysis.sharp_edges > 0);
        assert!(analysis.total_edges > 0);
    }

    #[test]
    fn extract_sharp() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let sharp = extract_sharp_edges_with_angle(&mesh, 30.0);
        assert!(sharp.lines.num_cells() > 0);
        assert!(sharp.cell_data().get_array("DihedralAngle").is_some());
    }

    #[test]
    fn display() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let analysis = analyze_edges(&mesh, 30.0);
        let s = format!("{analysis}");
        assert!(s.contains("Edges:"));
    }
}
