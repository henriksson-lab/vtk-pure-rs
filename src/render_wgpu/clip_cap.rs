//! Compute clip plane cap geometry.
//!
//! When a mesh is clipped by a plane (via fragment shader discard), the interior
//! is exposed. This module generates cap faces to close the cross-section.

use crate::render_wgpu::mesh::Vertex;

/// Generate cap geometry for a clip plane intersecting a triangle mesh.
///
/// For each triangle that straddles the clip plane, computes the intersection
/// edge.  The edges are assembled into contours and each contour is
/// triangulated as a fan from its own centroid.
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
    let mut segments: Vec<[[f32; 3]; 2]> = Vec::new();

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
            segments.push([isects[0], isects[1]]);
        }
    }

    if segments.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let contours = assemble_contours(&segments);

    for contour in contours {
        triangulate_contour(
            &contour,
            plane_normal,
            cap_color,
            &mut vertices,
            &mut indices,
        );
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

fn assemble_contours(segments: &[[[f32; 3]; 2]]) -> Vec<Vec<[f32; 3]>> {
    let mut unused = vec![true; segments.len()];
    let mut contours = Vec::new();

    for start_id in 0..segments.len() {
        if !unused[start_id] {
            continue;
        }

        unused[start_id] = false;
        let mut contour = vec![segments[start_id][0], segments[start_id][1]];

        loop {
            let tail = *contour.last().unwrap();
            let mut found = None;
            for (seg_id, segment) in segments.iter().enumerate() {
                if !unused[seg_id] {
                    continue;
                }
                if same_point(tail, segment[0]) {
                    found = Some((seg_id, segment[1]));
                    break;
                }
                if same_point(tail, segment[1]) {
                    found = Some((seg_id, segment[0]));
                    break;
                }
            }

            let Some((seg_id, next)) = found else {
                break;
            };
            unused[seg_id] = false;
            if same_point(next, contour[0]) {
                break;
            }
            contour.push(next);
        }

        contours.push(contour);
    }

    contours
}

fn triangulate_contour(
    contour: &[[f32; 3]],
    plane_normal: [f32; 3],
    cap_color: [f32; 3],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    if contour.len() < 2 {
        return;
    }

    let mut centroid = [0.0f32; 3];
    for p in contour {
        centroid[0] += p[0];
        centroid[1] += p[1];
        centroid[2] += p[2];
    }
    let inv_n = 1.0 / contour.len() as f32;
    centroid[0] *= inv_n;
    centroid[1] *= inv_n;
    centroid[2] *= inv_n;

    let start = vertices.len() as u32;
    vertices.push(Vertex {
        position: centroid,
        normal: plane_normal,
        color: cap_color,
        cell_id: 0,
    });
    for &position in contour {
        vertices.push(Vertex {
            position,
            normal: plane_normal,
            color: cap_color,
            cell_id: 0,
        });
    }

    let mut order: Vec<usize> = (0..contour.len()).collect();
    if contour.len() > 2 && projected_area(contour, plane_normal) < 0.0 {
        order.reverse();
    }

    if order.len() == 2 {
        indices.extend_from_slice(&[start, start + 1, start + 2]);
        return;
    }

    for i in 0..order.len() {
        let a = start + 1 + order[i] as u32;
        let b = start + 1 + order[(i + 1) % order.len()] as u32;
        indices.extend_from_slice(&[start, a, b]);
    }
}

fn projected_area(points: &[[f32; 3]], normal: [f32; 3]) -> f32 {
    let (u, v) = plane_basis(normal);
    let mut area = 0.0;
    for i in 0..points.len() {
        let p = points[i];
        let q = points[(i + 1) % points.len()];
        let px = dot3(p, u);
        let py = dot3(p, v);
        let qx = dot3(q, u);
        let qy = dot3(q, v);
        area += px * qy - qx * py;
    }
    area
}

fn plane_basis(normal: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let n = normalize3(normal);
    let axis = if n[0].abs() <= n[1].abs() && n[0].abs() <= n[2].abs() {
        [1.0, 0.0, 0.0]
    } else if n[1].abs() <= n[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let u = normalize3(cross3(axis, n));
    let v = cross3(n, u);
    (u, v)
}

fn same_point(a: [f32; 3], b: [f32; 3]) -> bool {
    const EPS: f32 = 1e-5;
    (a[0] - b[0]).abs() <= EPS && (a[1] - b[1]).abs() <= EPS && (a[2] - b[2]).abs() <= EPS
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = dot3(v, v).sqrt();
    if len <= f32::EPSILON {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
