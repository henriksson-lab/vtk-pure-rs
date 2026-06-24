//! Compute clip plane cap geometry.
//!
//! When a mesh is clipped by a plane (via fragment shader discard), the interior
//! is exposed. This module generates cap faces to close the cross-section.

use crate::render_wgpu::mesh::Vertex;

/// Generate cap geometry for a clip plane intersecting a triangle mesh.
///
/// For each triangle that straddles the clip plane, computes the intersection
/// edge. Collects all intersection edges and triangulates them as a fan from
/// their centroid to produce a cap surface.
///
/// Returns (vertices, indices) for the cap mesh, with normals set to the
/// clip plane normal.
pub fn generate_clip_cap(
    points: &[[f32; 3]],
    triangles: &[[u32; 3]],
    plane_normal: [f32; 3],
    plane_distance: f32,
    cap_color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let mut edge_points: Vec<[f32; 3]> = Vec::new();

    for tri in triangles {
        let p0 = points[tri[0] as usize];
        let p1 = points[tri[1] as usize];
        let p2 = points[tri[2] as usize];

        let d0 = dot3(plane_normal, p0) + plane_distance;
        let d1 = dot3(plane_normal, p1) + plane_distance;
        let d2 = dot3(plane_normal, p2) + plane_distance;

        // Find intersection points where the plane crosses triangle edges
        let mut isects = Vec::new();
        check_edge(p0, p1, d0, d1, &mut isects);
        check_edge(p1, p2, d1, d2, &mut isects);
        check_edge(p2, p0, d2, d0, &mut isects);

        // A plane-triangle intersection produces exactly 0 or 2 points
        if isects.len() == 2 {
            edge_points.push(isects[0]);
            edge_points.push(isects[1]);
        }
    }

    if edge_points.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // Compute centroid of all intersection points
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let n = edge_points.len() as f32;
    for p in &edge_points {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let centroid = [cx / n, cy / n, cz / n];

    // Build cap as fan triangles from centroid to each edge segment
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Add centroid as vertex 0
    vertices.push(Vertex {
        position: centroid,
        normal: plane_normal,
        color: cap_color,
    });

    for pair in edge_points.chunks_exact(2) {
        let base = vertices.len() as u32;
        vertices.push(Vertex {
            position: pair[0],
            normal: plane_normal,
            color: cap_color,
        });
        vertices.push(Vertex {
            position: pair[1],
            normal: plane_normal,
            color: cap_color,
        });
        // Triangle from centroid to edge
        indices.push(0); // centroid
        indices.push(base);
        indices.push(base + 1);
    }

    (vertices, indices)
}

/// Extract triangle data from a PolyData for clip cap computation.
pub fn extract_triangles(poly_data: &crate::data::PolyData) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut points = Vec::with_capacity(poly_data.points.len());
    for i in 0..poly_data.points.len() {
        let p = poly_data.points.get(i);
        points.push([p[0] as f32, p[1] as f32, p[2] as f32]);
    }

    let mut tris = Vec::new();
    for cell in poly_data.polys.iter() {
        if cell.len() >= 3 {
            // Fan triangulate
            for i in 1..cell.len() - 1 {
                tris.push([cell[0] as u32, cell[i] as u32, cell[i + 1] as u32]);
            }
        }
    }

    (points, tris)
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn check_edge(p0: [f32; 3], p1: [f32; 3], d0: f32, d1: f32, out: &mut Vec<[f32; 3]>) {
    if (d0 > 0.0) != (d1 > 0.0) {
        let t = d0 / (d0 - d1);
        out.push([
            p0[0] + t * (p1[0] - p0[0]),
            p0[1] + t * (p1[1] - p0[1]),
            p0[2] + t * (p1[2] - p0[2]),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_single_triangle() {
        // Triangle straddling XY plane (z=0)
        let points = vec![[0.0, 0.0, -1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
        let tris = vec![[0, 1, 2]];
        let (verts, idxs) = generate_clip_cap(
            &points,
            &tris,
            [0.0, 0.0, 1.0],
            0.0, // z=0 plane
            [1.0, 1.0, 1.0],
        );
        assert!(!verts.is_empty());
        assert!(!idxs.is_empty());
        // Should have 3 vertices (centroid + 2 edge points) and 3 indices (1 triangle)
        assert_eq!(verts.len(), 3);
        assert_eq!(idxs.len(), 3);
    }

    #[test]
    fn cap_no_intersection() {
        // Triangle entirely above plane
        let points = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 1.0, 3.0]];
        let tris = vec![[0, 1, 2]];
        let (verts, idxs) =
            generate_clip_cap(&points, &tris, [0.0, 0.0, 1.0], 0.0, [1.0, 1.0, 1.0]);
        assert!(verts.is_empty());
        assert!(idxs.is_empty());
    }
}
