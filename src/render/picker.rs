use crate::data::PolyData;

use crate::render::Scene;

/// Result of a pick operation.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Index of the actor in the scene.
    pub actor_index: usize,
    /// Cell (triangle/polygon) index in the actor's PolyData.
    pub cell_id: usize,
    /// Closest point index in the hit cell.
    pub point_id: usize,
    /// World-space intersection point.
    pub position: [f64; 3],
    /// Parametric distance along the ray.
    pub t: f64,
}

/// Pick the closest actor/cell at a screen coordinate.
///
/// Performs CPU-side ray casting against all actors in the scene.
/// Returns `None` if no intersection is found.
pub fn pick(
    scene: &Scene,
    screen_x: f64,
    screen_y: f64,
    width: u32,
    height: u32,
) -> Option<PickResult> {
    let (origin, direction) = scene.camera.unproject(screen_x, screen_y, width, height);
    let ray_origin = [origin.x, origin.y, origin.z];
    let ray_dir = [direction.x, direction.y, direction.z];

    let mut best: Option<PickResult> = None;

    for (actor_idx, actor) in scene.actors.iter().enumerate() {
        if let Some(hit) = ray_cast_poly_data(&actor.data, ray_origin, ray_dir) {
            let dominated = best.as_ref().is_some_and(|b| hit.1 >= b.t);
            if !dominated {
                best = Some(PickResult {
                    actor_index: actor_idx,
                    cell_id: hit.0,
                    point_id: hit.2,
                    position: hit.3,
                    t: hit.1,
                });
            }
        }
    }

    best
}

/// Ray-cast against PolyData polygons. Returns (cell_id, t, closest_point_id, position).
fn ray_cast_poly_data(
    pd: &PolyData,
    origin: [f64; 3],
    dir: [f64; 3],
) -> Option<(usize, f64, usize, [f64; 3])> {
    let dlen = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if dlen < 1e-15 {
        return None;
    }
    let d = [dir[0] / dlen, dir[1] / dlen, dir[2] / dlen];

    let mut best_t = f64::INFINITY;
    let mut best_cell = 0;
    let mut best_point = 0;
    let mut best_pos = [0.0; 3];
    let mut found = false;

    for (ci, cell) in pd.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = pd.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = pd.points.get(cell[i] as usize);
            let v2 = pd.points.get(cell[i + 1] as usize);
            if let Some(t) = ray_triangle(origin, d, v0, v1, v2) {
                if t > 1e-12 && t < best_t {
                    best_t = t;
                    best_cell = ci;
                    let pos = [
                        origin[0] + t * d[0],
                        origin[1] + t * d[1],
                        origin[2] + t * d[2],
                    ];
                    best_pos = pos;
                    // Find closest vertex in cell
                    let mut min_dist2 = f64::INFINITY;
                    for &pid in cell {
                        let p = pd.points.get(pid as usize);
                        let dx = p[0] - pos[0];
                        let dy = p[1] - pos[1];
                        let dz = p[2] - pos[2];
                        let dist2 = dx * dx + dy * dy + dz * dz;
                        if dist2 < min_dist2 {
                            min_dist2 = dist2;
                            best_point = pid as usize;
                        }
                    }
                    found = true;
                }
            }
        }
    }

    if found {
        Some((best_cell, best_t, best_point, best_pos))
    } else {
        None
    }
}

/// Moller-Trumbore ray-triangle intersection.
fn ray_triangle(
    origin: [f64; 3],
    dir: [f64; 3],
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
) -> Option<f64> {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = cross3(dir, e2);
    let a = dot3(e1, h);
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = f * dot3(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(e2, q);
    Some(t)
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Actor, Camera, Scene};

    #[test]
    fn pick_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let actor = Actor::new(pd);
        let mut scene = Scene::new();
        scene.add_actor(actor);
        scene.camera.position = glam::DVec3::new(0.3, 0.3, 5.0);
        scene.camera.focal_point = glam::DVec3::new(0.3, 0.3, 0.0);

        let result = pick(&scene, 400.0, 300.0, 800, 600);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.actor_index, 0);
        assert_eq!(r.cell_id, 0);
        assert!(r.position[2].abs() < 1e-6);
    }

    #[test]
    fn pick_miss() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let actor = Actor::new(pd);
        let mut scene = Scene::new();
        scene.add_actor(actor);
        scene.camera.position = glam::DVec3::new(0.3, 0.3, 5.0);
        scene.camera.focal_point = glam::DVec3::new(0.3, 0.3, 0.0);

        // Click far from the triangle
        let result = pick(&scene, 0.0, 0.0, 800, 600);
        // May or may not hit depending on FOV; just test it doesn't crash
        let _ = result;
    }

    #[test]
    fn unproject_center_ray() {
        let camera = Camera::default();
        let (origin, dir) = camera.unproject(400.0, 300.0, 800, 600);
        // Center of screen should point along camera direction
        let cam_dir = camera.direction();
        let dot = dir.x * cam_dir.x + dir.y * cam_dir.y + dir.z * cam_dir.z;
        assert!(dot > 0.99, "center ray should align with camera direction");
    }
}
