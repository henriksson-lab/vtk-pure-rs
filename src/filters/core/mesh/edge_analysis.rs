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
    let all_cells = surface_faces(mesh);
    let face_normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    let mut edge_map: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
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

    let (min_length, max_length) = if lengths.is_empty() {
        (0.0, 0.0)
    } else {
        (
            lengths.iter().cloned().fold(f64::MAX, f64::min),
            lengths.iter().cloned().fold(0.0f64, f64::max),
        )
    };
    let (min_dihedral, max_dihedral) = if dihedrals.is_empty() {
        (0.0, 0.0)
    } else {
        (
            dihedrals.iter().cloned().fold(f64::MAX, f64::min),
            dihedrals.iter().cloned().fold(0.0f64, f64::max),
        )
    };

    EdgeAnalysis {
        total_edges: total,
        boundary_edges: boundary,
        internal_edges: internal,
        non_manifold_edges: non_manifold,
        sharp_edges: sharp,
        min_length,
        max_length,
        mean_length: if total > 0 {
            total_len / total as f64
        } else {
            0.0
        },
        total_length: total_len,
        min_dihedral,
        max_dihedral,
        mean_dihedral: if !dihedrals.is_empty() {
            total_dih / dihedrals.len() as f64
        } else {
            0.0
        },
    }
}

/// Extract sharp edges as a line PolyData.
pub fn extract_sharp_edges_with_angle(mesh: &PolyData, sharp_angle_degrees: f64) -> PolyData {
    let all_cells = surface_faces(mesh);
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
            let a = cell[i];
            let b = cell[(i + 1) % nc];
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

fn surface_faces(mesh: &PolyData) -> Vec<Vec<usize>> {
    let n_points = mesh.points.len();
    let mut faces: Vec<Vec<usize>> = mesh
        .polys
        .iter()
        .filter_map(|c| valid_point_ids(c, n_points))
        .filter(|c| c.len() >= 2)
        .collect();

    for strip in mesh.strips.iter() {
        let Some(ids) = valid_point_ids(strip, n_points) else {
            continue;
        };
        for i in 0..ids.len().saturating_sub(2) {
            if i % 2 == 0 {
                faces.push(vec![ids[i], ids[i + 1], ids[i + 2]]);
            } else {
                faces.push(vec![ids[i + 1], ids[i], ids[i + 2]]);
            }
        }
    }

    faces
}

fn face_normal(mesh: &PolyData, cell: &[usize]) -> [f64; 3] {
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
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-15 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn valid_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
        .collect()
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

    #[test]
    fn triangle_strip_edges_are_analyzed() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let analysis = analyze_edges(&mesh, 30.0);

        assert_eq!(analysis.total_edges, 5);
        assert_eq!(analysis.boundary_edges, 4);
        assert_eq!(analysis.internal_edges, 1);
    }
}
