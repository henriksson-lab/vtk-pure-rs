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

    // Build vertex buffer (one vertex per line endpoint) so the VTK
    // attribute/cell id survives polygon-edge tessellation.
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut push_endpoint = |point_id: u32, cell_id: u32| -> u32 {
        let p = poly_data.points.get(point_id as usize);
        let idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: [p[0] as f32, p[1] as f32, p[2] as f32],
            normal: [0.0, 0.0, 1.0],
            color: point_colors[point_id as usize],
            cell_id,
        });
        idx
    };

    let line_cell_offset = poly_data.verts.num_cells();
    for (ci, cell) in poly_data.lines.iter().enumerate() {
        let cell_id = (line_cell_offset + ci) as u32;
        for segment in cell.windows(2) {
            let a = push_endpoint(segment[0] as u32, cell_id);
            let b = push_endpoint(segment[1] as u32, cell_id);
            indices.push(a);
            indices.push(b);
        }
    }

    let poly_cell_offset = poly_data.verts.num_cells() + poly_data.lines.num_cells();
    for (ci, cell) in poly_data.polys.iter().enumerate() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        let cell_id = (poly_cell_offset + ci) as u32;
        for i in 0..nc {
            let a = push_endpoint(cell[i] as u32, cell_id);
            let b = push_endpoint(cell[(i + 1) % nc] as u32, cell_id);
            indices.push(a);
            indices.push(b);
        }
    }

    let strip_cell_offset =
        poly_data.verts.num_cells() + poly_data.lines.num_cells() + poly_data.polys.num_cells();
    for (ci, cell) in poly_data.strips.iter().enumerate() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        let cell_id = (strip_cell_offset + ci) as u32;
        for edge_id in 0..nc {
            let (id1, id2) = if edge_id == 0 {
                (0, 1)
            } else if edge_id == nc - 1 {
                (edge_id - 1, edge_id)
            } else {
                (edge_id - 1, edge_id + 1)
            };
            let a = push_endpoint(cell[id1] as u32, cell_id);
            let b = push_endpoint(cell[id2] as u32, cell_id);
            indices.push(a);
            indices.push(b);
        }
    }

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_strip_wireframe_matches_vtk_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let (vertices, indices) = poly_data_to_wireframe(&pd, &Coloring::Solid([1.0, 1.0, 1.0]));

        assert_eq!(indices.len(), 8);
        let edges: Vec<([f32; 3], [f32; 3])> = indices
            .chunks_exact(2)
            .map(|edge| {
                (
                    vertices[edge[0] as usize].position,
                    vertices[edge[1] as usize].position,
                )
            })
            .collect();
        assert_eq!(
            edges,
            vec![
                ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
                ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
                ([1.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
                ([0.0, 1.0, 0.0], [1.0, 1.0, 0.0]),
            ]
        );
    }
}
