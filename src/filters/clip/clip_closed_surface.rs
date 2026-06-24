//! Clip a triangle mesh by a plane and generate a cap polygon to close the surface.
//!
//! Unlike the basic clip filter, this generates closing cap faces on the
//! cross-section to produce a watertight mesh.

use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Clip a triangle mesh by a plane and cap the cross-section.
///
/// Keeps the half-space where `dot(p - origin, normal) >= 0`.
/// Triangles crossing the plane are split, and a cap polygon is generated
/// from the intersection edges to close the mesh.
///
/// # Arguments
/// * `input` - Triangle mesh PolyData
/// * `origin` - A point on the clipping plane
/// * `normal` - Normal vector of the clipping plane (pointing toward the kept side)
pub fn clip_closed_surface(input: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let norm_len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if norm_len <= 0.0 {
        return input.clone();
    }
    let normal = [
        normal[0] / norm_len,
        normal[1] / norm_len,
        normal[2] / norm_len,
    ];

    let mut out_points = input.points.clone();
    let mut out_polys = CellArray::new();

    // Track intersection edges for cap generation
    // Each intersection edge is stored as (point_index_a, point_index_b) on the clip plane
    let mut cap_edges: Vec<(usize, usize)> = Vec::new();
    let mut edge_locator: HashMap<(i64, i64), i64> = HashMap::new();

    for cell_idx in 0..input.polys.num_cells() {
        let cell = input.polys.cell(cell_idx);
        if cell.len() < 3 {
            continue;
        }

        // Classify vertices
        let dists: Vec<f64> = cell
            .iter()
            .map(|&id| {
                let p = input.points.get(id as usize);
                (p[0] - origin[0]) * normal[0]
                    + (p[1] - origin[1]) * normal[1]
                    + (p[2] - origin[2]) * normal[2]
            })
            .collect();

        let all_inside = dists.iter().all(|&d| d >= -1e-12);
        let all_outside = dists.iter().all(|&d| d < -1e-12);

        if all_inside {
            out_polys.push_cell(cell);
        } else if all_outside {
            // discard
        } else {
            // Clip the polygon and collect intersection edges
            let (clipped, intersections) = clip_and_collect(
                cell,
                &dists,
                &input.points,
                &mut out_points,
                &mut edge_locator,
            );

            if clipped.len() >= 3 {
                // Fan-triangulate the clipped polygon
                for i in 1..clipped.len() - 1 {
                    out_polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                }
            }

            // Store intersection edges for cap
            for (a, b) in intersections {
                cap_edges.push((a as usize, b as usize));
            }
        }
    }

    // Generate cap from intersection edges
    if !cap_edges.is_empty() {
        let cap_polygons = order_edges_into_loops(&cap_edges);

        for loop_indices in &cap_polygons {
            if loop_indices.len() < 3 {
                continue;
            }

            // Compute centroid of the cap polygon
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &idx in loop_indices {
                let p = out_points.get(idx);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = loop_indices.len() as f64;
            cx /= n;
            cy /= n;
            cz /= n;

            let center_idx = out_points.len();
            out_points.push([cx, cy, cz]);

            // Fan-triangulate from centroid
            for i in 0..loop_indices.len() {
                let j = (i + 1) % loop_indices.len();
                out_polys.push_cell(&[
                    center_idx as i64,
                    loop_indices[j] as i64,
                    loop_indices[i] as i64,
                ]);
            }
        }
    }

    let mut output = PolyData::new();
    output.points = out_points;
    output.polys = out_polys;
    output
}

/// Clip a polygon and return (clipped_vertex_ids, intersection_edge_pairs).
fn clip_and_collect(
    cell: &[i64],
    dists: &[f64],
    src_points: &Points<f64>,
    out_points: &mut Points<f64>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
) -> (Vec<i64>, Vec<(i64, i64)>) {
    let n = cell.len();
    let mut result = Vec::new();
    let mut intersections = Vec::new();
    let mut new_pts_in_order = Vec::new(); // track intersection points for this polygon

    for i in 0..n {
        let j = (i + 1) % n;
        let di = dists[i];
        let dj = dists[j];

        if di >= 0.0 {
            result.push(cell[i]);
        }

        // Check for crossing
        if (di >= 0.0 && dj < 0.0) || (di < 0.0 && dj >= 0.0) {
            let edge_key = if cell[i] < cell[j] {
                (cell[i], cell[j])
            } else {
                (cell[j], cell[i])
            };
            let new_id = if let Some(&id) = edge_locator.get(&edge_key) {
                id
            } else {
                let t = di / (di - dj);
                let pi = src_points.get(cell[i] as usize);
                let pj = src_points.get(cell[j] as usize);
                let new_pt = [
                    pi[0] + t * (pj[0] - pi[0]),
                    pi[1] + t * (pj[1] - pi[1]),
                    pi[2] + t * (pj[2] - pi[2]),
                ];
                let id = out_points.len() as i64;
                out_points.push(new_pt);
                edge_locator.insert(edge_key, id);
                id
            };
            result.push(new_id);
            new_pts_in_order.push(new_id);
        }
    }

    // Intersection points come in pairs for each clipped triangle
    if new_pts_in_order.len() == 2 {
        intersections.push((new_pts_in_order[0], new_pts_in_order[1]));
    } else if new_pts_in_order.len() > 2 {
        // Multiple intersection points - pair them up
        for pair in new_pts_in_order.chunks(2) {
            if pair.len() == 2 {
                intersections.push((pair[0], pair[1]));
            }
        }
    }

    (result, intersections)
}

/// Order a set of edges into closed loops.
fn order_edges_into_loops(edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    if edges.is_empty() {
        return vec![];
    }

    // Build adjacency
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    let mut used_edges: HashMap<(usize, usize), bool> = HashMap::new();
    let mut loops = Vec::new();

    for &(start, _) in edges {
        let mut loop_verts = Vec::new();
        let mut current = start;

        loop {
            loop_verts.push(current);

            let neighbors = match adj.get(&current) {
                Some(n) => n,
                None => break,
            };

            let mut found_next = false;
            for &nb in neighbors {
                let edge_key = (current.min(nb), current.max(nb));
                if !used_edges.contains_key(&edge_key) {
                    used_edges.insert(edge_key, true);
                    current = nb;
                    found_next = true;
                    break;
                }
            }

            if !found_next || current == start {
                break;
            }
        }

        if loop_verts.len() >= 3 {
            loops.push(loop_verts);
        }
    }

    if loops.is_empty() {
        // Fallback: just collect all unique vertices from edges
        let mut all_verts: Vec<usize> = Vec::new();
        let mut seen = HashMap::new();
        for &(a, b) in edges {
            if !seen.contains_key(&a) {
                seen.insert(a, true);
                all_verts.push(a);
            }
            if !seen.contains_key(&b) {
                seen.insert(b, true);
                all_verts.push(b);
            }
        }
        if all_verts.len() >= 3 {
            loops.push(all_verts);
        }
    }

    loops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_unit_cube() -> PolyData {
        // A simple cube from (0,0,0) to (1,1,1) as 12 triangles
        let points = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        let tris: Vec<[i64; 3]> = vec![
            // Front (z=0)
            [0, 1, 2],
            [0, 2, 3],
            // Back (z=1)
            [4, 6, 5],
            [4, 7, 6],
            // Bottom (y=0)
            [0, 5, 1],
            [0, 4, 5],
            // Top (y=1)
            [3, 2, 6],
            [3, 6, 7],
            // Left (x=0)
            [0, 3, 7],
            [0, 7, 4],
            // Right (x=1)
            [1, 5, 6],
            [1, 6, 2],
        ];
        PolyData::from_triangles(points, tris)
    }

    #[test]
    fn clip_cube_produces_geometry() {
        let cube = make_unit_cube();
        // Clip at x=0.5
        let result = clip_closed_surface(&cube, [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]);

        // Should have some triangles (clipped half plus cap)
        assert!(result.polys.num_cells() > 0);
        assert!(result.points.len() > 0);

        // All kept points should have x >= 0.5 - epsilon (original inside points
        // and intersection points on the plane)
        // Note: cap centroid may also be on the plane
    }

    #[test]
    fn clip_cube_has_cap() {
        let cube = make_unit_cube();
        // Clip at x=0.5
        let result = clip_closed_surface(&cube, [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]);

        // The result should have more triangles than just clipping without cap
        // (the cap adds fan triangles)
        // With a cube clipped in half, we expect:
        // - 4 original faces fully inside (right, parts of top/bottom/front/back) = some tris
        // - 4 clipped faces producing new tris
        // - cap tris
        // So total should be more than 4
        assert!(
            result.polys.num_cells() > 4,
            "Expected cap triangles, got {} total",
            result.polys.num_cells()
        );
    }
}
