use crate::data::PolyData;
use crate::render::Coloring;

use crate::render_wgpu::mesh::{resolve_colors_pub, Vertex};

/// Convert PolyData polygon edges to line-list vertices and indices
/// suitable for wireframe rendering.
///
/// Returns vertex and index buffers where each edge of each polygon
/// becomes a pair of indices.
pub fn poly_data_to_wireframe(
    poly_data: &PolyData,
    coloring: &Coloring,
) -> (Vec<Vertex>, Vec<u32>) {
    let point_colors = resolve_colors_pub(poly_data, coloring);
    let n = poly_data.points.len();

    // Build vertex buffer (one vertex per point)
    let mut vertices: Vec<Vertex> = Vec::with_capacity(n);
    for (i, color) in point_colors.iter().enumerate() {
        let p = poly_data.points.get(i);
        vertices.push(Vertex {
            position: [p[0] as f32, p[1] as f32, p[2] as f32],
            normal: [0.0, 0.0, 1.0],
            color: *color,
        });
    }

    // Build index buffer (line list)
    let mut indices: Vec<u32> = Vec::new();
    for cell in poly_data.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            indices.push(cell[i] as u32);
            indices.push(cell[(i + 1) % nc] as u32);
        }
    }

    (vertices, indices)
}
