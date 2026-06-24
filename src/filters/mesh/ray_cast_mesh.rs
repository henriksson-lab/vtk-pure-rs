//! Ray-mesh intersection queries.

use crate::data::PolyData;

/// Ray-mesh intersection result.
pub struct RayHit {
    pub point: [f64; 3],
    pub t: f64,
    pub cell_index: usize,
    pub u: f64,
    pub v: f64,
}

/// Cast a ray and find all intersections with the mesh, sorted by distance.
pub fn ray_cast_all(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Vec<RayHit> {
    let mut hits = Vec::new();
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if let Some((t, u, v)) = ray_triangle(origin, direction, a, b, c) {
                hits.push(RayHit {
                    point: [
                        origin[0] + t * direction[0],
                        origin[1] + t * direction[1],
                        origin[2] + t * direction[2],
                    ],
                    t,
                    cell_index: ci,
                    u,
                    v,
                });
            }
        }
    }
    hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    hits
}

/// Cast a ray and find the first intersection.
pub fn ray_cast_first(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Option<RayHit> {
    let hits = ray_cast_all(mesh, origin, direction);
    hits.into_iter().next()
}

/// Check if a ray intersects the mesh at all.
pub fn ray_intersects(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> bool {
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if ray_triangle(origin, direction, a, b, c).is_some() {
                return true;
            }
        }
    }
    false
}

fn ray_triangle(
    o: [f64; 3],
    d: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> Option<(f64, f64, f64)> {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let h = cross(d, e2);
    let det = dot(e1, h);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = [o[0] - a[0], o[1] - a[1], o[2] - a[2]];
    let u = inv * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = cross(s, e1);
    let v = inv * dot(d, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv * dot(e2, q);
    if t > 1e-12 {
        Some((t, u, v))
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hit() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let hit = ray_cast_first(&mesh, [1.0, 1.0, 5.0], [0.0, 0.0, -1.0]);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.point[2] - 0.0).abs() < 1e-10);
        assert!((h.t - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_miss() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(!ray_intersects(&mesh, [10.0, 10.0, 5.0], [0.0, 0.0, -1.0]));
    }
    #[test]
    fn test_all() {
        // Two parallel triangles
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [0.0, 0.0, 3.0],
                [2.0, 0.0, 3.0],
                [1.0, 2.0, 3.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let hits = ray_cast_all(&mesh, [1.0, 1.0, 10.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 2);
        assert!(hits[0].t < hits[1].t);
    }
}
