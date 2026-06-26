use crate::data::PolyData;

/// Result of a ray-triangle intersection.
#[derive(Debug, Clone)]
pub struct RayHit {
    /// Distance along the ray direction.
    pub t: f64,
    /// World-space intersection point.
    pub point: [f64; 3],
    /// Index of the cell (triangle) that was hit.
    pub cell_id: usize,
    /// Geometric normal of the hit triangle.
    pub normal: [f64; 3],
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len: f64 = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-30 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Moller-Trumbore ray-triangle intersection for a single triangle.
/// Returns Some(t) if the ray hits, None otherwise.
fn ray_triangle(
    origin: [f64; 3],
    direction: [f64; 3],
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
) -> Option<f64> {
    let eps: f64 = 1e-12;
    let e1 = sub(v1, v0);
    let e2 = sub(v2, v0);
    let h = cross(direction, e2);
    let a: f64 = dot(e1, h);
    if a > -eps && a < eps {
        return None;
    }
    let f: f64 = 1.0 / a;
    let s = sub(origin, v0);
    let u: f64 = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = cross(s, e1);
    let v: f64 = f * dot(direction, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t: f64 = f * dot(e2, q);
    if t > eps {
        Some(t)
    } else {
        None
    }
}

/// Find all ray-triangle intersections in a PolyData mesh.
///
/// Uses the Moller-Trumbore algorithm. Only triangular cells in `polys`
/// are tested. Results are sorted by increasing `t`.
pub fn ray_intersect_mesh(origin: [f64; 3], direction: [f64; 3], mesh: &PolyData) -> Vec<RayHit> {
    let mut hits: Vec<RayHit> = Vec::new();
    let dir = normalize(direction);

    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        // Fan-triangulate polygons with more than 3 vertices
        let v0 = mesh.points.get(cell[0] as usize);
        let mut cell_hit: Option<RayHit> = None;
        for tri in 1..cell.len() - 1 {
            let v1 = mesh.points.get(cell[tri] as usize);
            let v2 = mesh.points.get(cell[tri + 1] as usize);
            if let Some(t) = ray_triangle(origin, dir, v0, v1, v2) {
                let hit = match &cell_hit {
                    Some(existing) => t < existing.t,
                    None => true,
                };
                if hit {
                    let point = [
                        origin[0] + t * dir[0],
                        origin[1] + t * dir[1],
                        origin[2] + t * dir[2],
                    ];
                    let e1 = sub(v1, v0);
                    let e2 = sub(v2, v0);
                    let normal = normalize(cross(e1, e2));
                    cell_hit = Some(RayHit {
                        t,
                        point,
                        cell_id,
                        normal,
                    });
                }
            }
        }
        if let Some(hit) = cell_hit {
            hits.push(hit);
        }
    }

    hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_triangle() -> PolyData {
        PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn ray_hits_triangle() {
        let mesh = make_triangle();
        let hits = ray_intersect_mesh([0.5, 0.5, 1.0], [0.0, 0.0, -1.0], &mesh);
        assert_eq!(hits.len(), 1);
        assert!((hits[0].t - 1.0).abs() < 1e-9);
        assert!((hits[0].point[2]).abs() < 1e-9);
        assert_eq!(hits[0].cell_id, 0);
        // Normal should point in +Z or -Z direction
        assert!((hits[0].normal[2].abs() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ray_misses_triangle() {
        let mesh = make_triangle();
        let hits = ray_intersect_mesh([5.0, 5.0, 1.0], [0.0, 0.0, -1.0], &mesh);
        assert!(hits.is_empty());
    }

    #[test]
    fn ray_hits_two_triangles_sorted() {
        let mesh = PolyData::from_triangles(
            vec![
                // triangle at z=0
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                // triangle at z=3
                [0.0, 0.0, 3.0],
                [2.0, 0.0, 3.0],
                [0.0, 2.0, 3.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let hits = ray_intersect_mesh([0.5, 0.5, 10.0], [0.0, 0.0, -1.0], &mesh);
        assert_eq!(hits.len(), 2);
        assert!(hits[0].t < hits[1].t);
        assert!((hits[0].point[2] - 3.0).abs() < 1e-9);
        assert!((hits[1].point[2]).abs() < 1e-9);
    }

    #[test]
    fn polygon_fan_internal_edge_counts_as_one_cell_hit() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let hits = ray_intersect_mesh([0.5, 0.5, 1.0], [0.0, 0.0, -1.0], &mesh);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_id, 0);
    }
}
