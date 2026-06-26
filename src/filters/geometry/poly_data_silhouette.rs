//! View-dependent silhouette edge extraction.
//!
//! Extracts edges where one adjacent face is front-facing and the other
//! is back-facing relative to a view direction.

use crate::data::{CellArray, Points, PolyData};

/// Options matching vtkPolyDataSilhouette defaults where practical.
#[derive(Debug, Clone, Copy)]
pub struct SilhouetteOptions {
    pub enable_feature_angle: bool,
    pub feature_angle_degrees: f64,
    pub border_edges: bool,
}

impl Default for SilhouetteOptions {
    fn default() -> Self {
        Self {
            enable_feature_angle: true,
            feature_angle_degrees: 60.0,
            border_edges: false,
        }
    }
}

/// Extract silhouette edges relative to a view direction.
///
/// A silhouette edge is shared by one front-facing and one back-facing
/// triangle. Sharp feature edges are included by default, and boundary edges
/// are omitted by default, matching vtkPolyDataSilhouette defaults.
pub fn silhouette_edges(mesh: &PolyData, view_dir: [f64; 3]) -> PolyData {
    silhouette_edges_with_options(mesh, view_dir, SilhouetteOptions::default())
}

/// Extract silhouette edges with explicit feature-angle and border-edge options.
pub fn silhouette_edges_with_options(
    mesh: &PolyData,
    view_dir: [f64; 3],
    options: SilhouetteOptions,
) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return PolyData::new();
    }

    // Compute face normals
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let face_normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return [0.0, 0.0, 1.0];
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ]
        })
        .collect();

    // Build edge → face adjacency
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let edge = (a.min(b), a.max(b));
            edge_faces.entry(edge).or_default().push(ci);
        }
    }

    let feature_angle_cos = options.feature_angle_degrees.to_radians().cos();

    // Find silhouette edges
    let mut sil_points = Points::<f64>::new();
    let mut sil_lines = CellArray::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for ((a, b), faces) in &edge_faces {
        let is_silhouette = if faces.len() == 1 {
            options.border_edges
        } else if faces.len() == 2 {
            let d0 = dot(face_normals[faces[0]], view_dir);
            let d1 = dot(face_normals[faces[1]], view_dir);
            let edge_angle_cos = normalized_dot(face_normals[faces[0]], face_normals[faces[1]]);
            (d0 * d1) < 0.0 || (options.enable_feature_angle && edge_angle_cos < feature_angle_cos)
        } else {
            false
        };

        if is_silhouette {
            let ia = *point_map.entry(*a).or_insert_with(|| {
                let idx = sil_points.len();
                sil_points.push(mesh.points.get(*a));
                idx
            });
            let ib = *point_map.entry(*b).or_insert_with(|| {
                let idx = sil_points.len();
                sil_points.push(mesh.points.get(*b));
                idx
            });
            sil_lines.push_cell(&[ia as i64, ib as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = sil_points;
    result.lines = sil_lines;
    result
}

/// Extract silhouette edges relative to a camera position (perspective).
pub fn silhouette_edges_perspective(mesh: &PolyData, camera_pos: [f64; 3]) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return PolyData::new();
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    // Per-face: compute normal and face center, then determine front/back
    let mut face_front: Vec<bool> = Vec::with_capacity(n_cells);
    for cell in &all_cells {
        if cell.len() < 3 {
            face_front.push(true);
            continue;
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
        // View vector from face center to camera
        let cx = (a[0] + b[0] + c[0]) / 3.0;
        let cy = (a[1] + b[1] + c[1]) / 3.0;
        let cz = (a[2] + b[2] + c[2]) / 3.0;
        let view = [camera_pos[0] - cx, camera_pos[1] - cy, camera_pos[2] - cz];
        face_front.push(dot(n, view) > 0.0);
    }

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

    let mut sil_points = Points::<f64>::new();
    let mut sil_lines = CellArray::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for ((a, b), faces) in &edge_faces {
        let is_sil = if faces.len() == 1 {
            false
        } else if faces.len() == 2 {
            face_front[faces[0]] != face_front[faces[1]]
        } else {
            false
        };

        if is_sil {
            let ia = *point_map.entry(*a).or_insert_with(|| {
                let idx = sil_points.len();
                sil_points.push(mesh.points.get(*a));
                idx
            });
            let ib = *point_map.entry(*b).or_insert_with(|| {
                let idx = sil_points.len();
                sil_points.push(mesh.points.get(*b));
                idx
            });
            sil_lines.push_cell(&[ia as i64, ib as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = sil_points;
    result.lines = sil_lines;
    result
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalized_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    let la = dot(a, a).sqrt();
    let lb = dot(b, b).sqrt();
    if la <= 1e-15 || lb <= 1e-15 {
        1.0
    } else {
        dot(a, b) / (la * lb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_silhouette() {
        // Front face and back face of a simple mesh
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
                [0, 1, 2],
                [0, 2, 3], // front (z=0)
                [4, 6, 5],
                [4, 7, 6], // back (z=1, reversed winding)
                [0, 4, 5],
                [0, 5, 1], // bottom
                [2, 6, 7],
                [2, 7, 3], // top
            ],
        );
        let sil = silhouette_edges(&mesh, [0.0, 0.0, -1.0]);
        assert!(sil.lines.num_cells() > 0, "should have silhouette edges");
    }

    #[test]
    fn single_triangle_default_omits_boundary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sil = silhouette_edges(&mesh, [0.0, 0.0, 1.0]);
        assert_eq!(sil.lines.num_cells(), 0);
    }

    #[test]
    fn single_triangle_border_edges() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sil = silhouette_edges_with_options(
            &mesh,
            [0.0, 0.0, 1.0],
            SilhouetteOptions {
                border_edges: true,
                ..SilhouetteOptions::default()
            },
        );
        assert_eq!(sil.lines.num_cells(), 3); // all boundary edges
    }

    #[test]
    fn perspective_silhouette() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sil = silhouette_edges_perspective(&mesh, [0.5, 0.5, 5.0]);
        assert_eq!(sil.lines.num_cells(), 0);
    }

    #[test]
    fn empty_mesh() {
        let sil = silhouette_edges(&PolyData::new(), [0.0, 0.0, 1.0]);
        assert_eq!(sil.lines.num_cells(), 0);
    }
}
