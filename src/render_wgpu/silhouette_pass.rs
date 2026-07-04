use crate::data::PolyData;
use crate::render::SilhouetteConfig;

use crate::render_wgpu::mesh::Vertex;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Default)]
struct EdgeNormals {
    left: Option<[f64; 3]>,
    right: Option<[f64; 3]>,
    cell_id: u32,
}

/// Extract silhouette edges from a PolyData mesh viewed from a camera position.
///
/// A silhouette edge is one where one adjacent face is front-facing and the other
/// is back-facing relative to the view direction. Also includes boundary edges.
/// Returns line-list vertices and indices for rendering with the wireframe pipeline.
pub fn extract_silhouette_edges(
    pd: &PolyData,
    camera_pos: [f64; 3],
    config: &SilhouetteConfig,
) -> (Vec<Vertex>, Vec<u32>) {
    let color = config.color;
    let n_pts = pd.points.len();
    if n_pts == 0 {
        return (Vec::new(), Vec::new());
    }

    // Build ordered edge normals following vtkPolyDataSilhouette's left/right
    // normal cache for vtkOrderedEdge.
    let mut edges: BTreeMap<(usize, usize), EdgeNormals> = BTreeMap::new();

    for (ci, cell) in pd.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }

        let normal = polygon_normal(pd, cell);

        // Register edges
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            let edge = edges.entry(key).or_default();
            edge.cell_id = ci as u32;
            if a < b {
                edge.left = Some(normal);
            } else {
                edge.right = Some(normal);
            }
        }
    }

    // Extract silhouette edges
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (&(a, b), edge) in &edges {
        let pa = pd.points.get(a);
        let pb = pd.points.get(b);
        let edge_center = [
            0.5 * (pa[0] + pb[0]),
            0.5 * (pa[1] + pb[1]),
            0.5 * (pa[2] + pb[2]),
        ];
        let view_dir = [
            camera_pos[0] - edge_center[0],
            camera_pos[1] - edge_center[1],
            camera_pos[2] - edge_center[2],
        ];

        let is_silhouette = match (edge.left, edge.right) {
            (Some(left), Some(right)) => {
                let d1 = dot3(view_dir, left);
                let d2 = dot3(view_dir, right);
                let edge_angle_cos = dot3(left, right);
                (d1 * d2) < 0.0 || edge_angle_cos < config.normal_threshold as f64
            }
            // VTK only outputs border edges when BorderEdges is enabled. The
            // render config has no exposed switch, so preserve the existing
            // outline behavior as BorderEdges-on.
            (Some(normal), None) | (None, Some(normal)) => {
                if config.normal_threshold > -1.0 {
                    dot3(normal, normal) > 0.25
                } else {
                    true
                }
            }
            (None, None) => false,
        };

        if is_silhouette {
            let base = vertices.len() as u32;
            vertices.push(Vertex {
                position: [pa[0] as f32, pa[1] as f32, pa[2] as f32],
                normal: [0.0, 0.0, 1.0],
                color,
                cell_id: edge.cell_id,
            });
            vertices.push(Vertex {
                position: [pb[0] as f32, pb[1] as f32, pb[2] as f32],
                normal: [0.0, 0.0, 1.0],
                color,
                cell_id: edge.cell_id,
            });
            indices.push(base);
            indices.push(base + 1);
        }
    }

    (vertices, indices)
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn polygon_normal(pd: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut normal = [0.0; 3];
    for i in 0..cell.len() {
        let Some(a) = valid_point_index(cell[i], pd.points.len()) else {
            return [0.0, 0.0, 1.0];
        };
        let Some(b) = valid_point_index(cell[(i + 1) % cell.len()], pd.points.len()) else {
            return [0.0, 0.0, 1.0];
        };
        let p = pd.points.get(a);
        let q = pd.points.get(b);
        normal[0] += (p[1] - q[1]) * (p[2] + q[2]);
        normal[1] += (p[2] - q[2]) * (p[0] + q[0]);
        normal[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }

    let len = dot3(normal, normal).sqrt();
    if len > 1e-12 {
        [normal[0] / len, normal[1] / len, normal[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silhouette_of_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let config = SilhouetteConfig {
            color: [0.0, 0.0, 0.0],
            enabled: true,
            ..Default::default()
        };
        let (verts, idxs) = extract_silhouette_edges(&pd, [0.5, 0.5, 5.0], &config);
        // Single triangle: all 3 edges are boundary edges, all silhouette
        assert_eq!(idxs.len(), 6); // 3 edges * 2 indices
        assert_eq!(verts.len(), 6); // 3 edges * 2 vertices
    }

    #[test]
    fn silhouette_of_two_coplanar_triangles() {
        // Two triangles sharing edge 1-2, same plane
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let config = SilhouetteConfig {
            color: [0.0, 0.0, 0.0],
            enabled: true,
            ..Default::default()
        };
        let (_, idxs) = extract_silhouette_edges(&pd, [0.5, 0.0, 5.0], &config);
        // Shared edge 0-1 is NOT silhouette (both faces front-facing)
        // 4 boundary edges are silhouette
        assert_eq!(idxs.len(), 8); // 4 edges * 2
    }

    #[test]
    fn silhouette_uses_whole_polygon_for_normal() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let normal = polygon_normal(&pd, pd.polys.cell(0));
        assert!((normal[2] - 1.0).abs() < 1e-12);
    }
}
