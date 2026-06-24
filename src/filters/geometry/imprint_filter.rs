//! Simple imprint filter: project points and edges from mesh B onto mesh A.
//!
//! For each point of B, finds the nearest point on A's surface and adds it as
//! a new point. Edges of B are projected as connecting edges between the
//! imprinted points.

use crate::data::{Points, PolyData};

/// Imprint the edges of mesh B onto mesh A's surface.
///
/// For each point of mesh B, the nearest point on the surface of mesh A is
/// computed, and that projection is added as a new point in A. Edges from B
/// are then added as line cells connecting the projected points.
///
/// Returns a new PolyData containing A's original geometry plus the imprinted
/// points and edges.
pub fn imprint(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    let mut out_points = mesh_a.points.clone();
    let out_polys = mesh_a.polys.clone();
    let mut out_lines = mesh_a.lines.clone();

    // Collect all triangles from A for closest-point queries
    let triangles: Vec<[usize; 3]> = (0..mesh_a.polys.num_cells())
        .filter_map(|i| {
            let cell = mesh_a.polys.cell(i);
            if cell.len() >= 3 {
                Some([cell[0] as usize, cell[1] as usize, cell[2] as usize])
            } else {
                None
            }
        })
        .collect();

    // For each point of B, find nearest point on A's surface
    let mut projected_indices: Vec<usize> = Vec::with_capacity(mesh_b.points.len());

    for i in 0..mesh_b.points.len() {
        let bp = mesh_b.points.get(i);
        let proj = closest_point_on_mesh(&mesh_a.points, &triangles, bp);
        let idx = out_points.len();
        out_points.push(proj);
        projected_indices.push(idx);
    }

    // Add edges from B as line cells using projected indices
    for cell in mesh_b.polys.iter() {
        if cell.len() >= 2 {
            for w in cell.windows(2) {
                let a = w[0] as usize;
                let b = w[1] as usize;
                if a < projected_indices.len() && b < projected_indices.len() {
                    out_lines
                        .push_cell(&[projected_indices[a] as i64, projected_indices[b] as i64]);
                }
            }
            // Close the polygon edge loop
            let first = cell[0] as usize;
            let last = cell[cell.len() - 1] as usize;
            if first < projected_indices.len() && last < projected_indices.len() {
                out_lines.push_cell(&[
                    projected_indices[last] as i64,
                    projected_indices[first] as i64,
                ]);
            }
        }
    }

    // Also add line cells from B
    for cell in mesh_b.lines.iter() {
        let mapped: Vec<i64> = cell
            .iter()
            .filter_map(|&id| {
                let uid = id as usize;
                if uid < projected_indices.len() {
                    Some(projected_indices[uid] as i64)
                } else {
                    None
                }
            })
            .collect();
        if mapped.len() >= 2 {
            out_lines.push_cell(&mapped);
        }
    }

    let mut output = PolyData::new();
    output.points = out_points;
    output.polys = out_polys;
    output.lines = out_lines;
    output
}

/// Find the closest point on a triangle mesh to the given query point.
fn closest_point_on_mesh(
    points: &Points<f64>,
    triangles: &[[usize; 3]],
    query: [f64; 3],
) -> [f64; 3] {
    let mut best_dist = f64::MAX;
    let mut best_point = query;

    for tri in triangles {
        let a = points.get(tri[0]);
        let b = points.get(tri[1]);
        let c = points.get(tri[2]);
        let cp = closest_point_on_triangle(a, b, c, query);
        let d = dist_sq(cp, query);
        if d < best_dist {
            best_dist = d;
            best_point = cp;
        }
    }

    // If no triangles, find nearest vertex
    if triangles.is_empty() {
        for i in 0..points.len() {
            let p = points.get(i);
            let d = dist_sq(p, query);
            if d < best_dist {
                best_dist = d;
                best_point = p;
            }
        }
    }

    best_point
}

/// Closest point on a triangle to a query point.
fn closest_point_on_triangle(a: [f64; 3], b: [f64; 3], c: [f64; 3], p: [f64; 3]) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add(a, scale(ab, v));
    }

    let cp_vec = sub(p, c);
    let d5 = dot(ab, cp_vec);
    let d6 = dot(ac, cp_vec);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add(a, scale(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add(b, scale(sub(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add(a, add(scale(ab, v), scale(ac, w)))
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imprint_point_onto_triangle() {
        // Mesh A: single triangle on z=0 plane
        let mesh_a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // Mesh B: a small triangle slightly above A
        let mesh_b = PolyData::from_triangles(
            vec![[0.5, 0.5, 0.1], [1.5, 0.5, 0.1], [1.0, 1.0, 0.1]],
            vec![[0, 1, 2]],
        );

        let result = imprint(&mesh_a, &mesh_b);

        // Should have A's 3 points + B's 3 projected points
        assert_eq!(result.points.len(), 6);
        // A's original triangle should still be there
        assert_eq!(result.polys.num_cells(), 1);
        // Edges from B's triangle should be imprinted as lines
        assert!(result.lines.num_cells() > 0);

        // Projected points should be on z=0 plane (projected onto A)
        for i in 3..6 {
            let p = result.points.get(i);
            assert!(p[2].abs() < 1e-10, "projected point should be on z=0");
        }
    }
}
