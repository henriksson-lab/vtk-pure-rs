//! Impostor rendering support.
//!
//! Replaces distant actors with camera-facing quads (billboards) for
//! faster rendering of complex scenes.

use crate::data::{CellArray, Points, PolyData};

use crate::render::scene::Actor;
use crate::render::Camera;

/// Configuration for impostor-based level-of-detail rendering.
#[derive(Debug, Clone)]
pub struct ImpostorConfig {
    /// Whether impostor rendering is enabled.
    pub enabled: bool,
    /// Distance from camera beyond which actors are replaced with impostors.
    pub distance_threshold: f64,
    /// Resolution of the impostor texture in pixels.
    pub resolution: u32,
}

impl Default for ImpostorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            distance_threshold: 100.0,
            resolution: 256,
        }
    }
}

/// Generate impostor quads for actors that are beyond the distance threshold.
///
/// Returns a list of (actor_index, impostor_quad) pairs for actors that
/// should be replaced with impostors.
pub fn generate_impostor_quads(actors: &[Actor], camera: &Camera) -> Vec<(usize, PolyData)> {
    let config = ImpostorConfig {
        enabled: true,
        ..Default::default()
    };
    generate_impostor_quads_with_config(actors, camera, &config)
}

/// Generate impostor quads using the supplied impostor configuration.
pub fn generate_impostor_quads_with_config(
    actors: &[Actor],
    camera: &Camera,
    config: &ImpostorConfig,
) -> Vec<(usize, PolyData)> {
    if !config.enabled {
        return Vec::new();
    }

    let cam_pos = [camera.position.x, camera.position.y, camera.position.z];
    let cam_right = {
        let fwd = camera.direction();
        let up = camera.view_up;
        let r = fwd.cross(up);
        let r = if r.length_squared() > 1e-24 {
            r.normalize()
        } else {
            camera.right()
        };
        [r.x, r.y, r.z]
    };
    let up = camera.up();
    let cam_up = [up.x, up.y, up.z];

    let mut result = Vec::new();

    for (i, actor) in actors.iter().enumerate() {
        if !actor.visible {
            continue;
        }
        let (center, size) = actor_impostor_bounds(actor);
        let dx = center[0] - cam_pos[0];
        let dy = center[1] - cam_pos[1];
        let dz = center[2] - cam_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < config.distance_threshold {
            continue;
        }

        let quad = billboard_quad(center, size, cam_right, cam_up);
        result.push((i, quad));
    }

    result
}

fn actor_impostor_bounds(actor: &Actor) -> ([f64; 3], f64) {
    let bounds = actor.data.points.bounds();
    if bounds.is_empty() {
        return (actor.position, actor.scale.abs());
    }

    let center = bounds.center();
    let scale = actor.scale.abs();
    let size = bounds.diagonal_length() * scale;
    (
        [
            actor.position[0] + center[0] * actor.scale,
            actor.position[1] + center[1] * actor.scale,
            actor.position[2] + center[2] * actor.scale,
        ],
        if size > 0.0 { size } else { scale },
    )
}

/// Create a single camera-facing quad (billboard) as PolyData.
///
/// The quad is centered at `center` with the given `size`, oriented
/// using `camera_right` and `camera_up` vectors.
pub fn billboard_quad(
    center: [f64; 3],
    size: f64,
    camera_right: [f64; 3],
    camera_up: [f64; 3],
) -> PolyData {
    let half = size * 0.5;
    let camera_right = normalize_or(camera_right, [1.0, 0.0, 0.0]);
    let camera_up = normalize_or(camera_up, [0.0, 1.0, 0.0]);

    // Four corners: center +/- half*right +/- half*up
    let corners = [
        [
            center[0] - half * camera_right[0] - half * camera_up[0],
            center[1] - half * camera_right[1] - half * camera_up[1],
            center[2] - half * camera_right[2] - half * camera_up[2],
        ],
        [
            center[0] + half * camera_right[0] - half * camera_up[0],
            center[1] + half * camera_right[1] - half * camera_up[1],
            center[2] + half * camera_right[2] - half * camera_up[2],
        ],
        [
            center[0] + half * camera_right[0] + half * camera_up[0],
            center[1] + half * camera_right[1] + half * camera_up[1],
            center[2] + half * camera_right[2] + half * camera_up[2],
        ],
        [
            center[0] - half * camera_right[0] + half * camera_up[0],
            center[1] - half * camera_right[1] + half * camera_up[1],
            center[2] - half * camera_right[2] + half * camera_up[2],
        ],
    ];

    let mut points = Points::<f64>::new();
    for c in &corners {
        points.push(*c);
    }

    let mut polys = CellArray::new();
    polys.push_cell(&[0, 1, 2, 3]);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

fn normalize_or(v: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let len2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len2 > 1e-24 {
        let inv_len = 1.0 / len2.sqrt();
        [v[0] * inv_len, v[1] * inv_len, v[2] * inv_len]
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_billboard_quad() {
        let quad = billboard_quad([0.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(quad.points.len(), 4);
        assert_eq!(quad.polys.num_cells(), 1);

        // Check that opposite corners are 2*sqrt(2) apart (diagonal of 2x2 quad)
        let p0 = quad.points.get(0);
        let p2 = quad.points.get(2);
        let diag =
            ((p2[0] - p0[0]).powi(2) + (p2[1] - p0[1]).powi(2) + (p2[2] - p0[2]).powi(2)).sqrt();
        assert!((diag - 2.0_f64 * 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_impostor_config_default() {
        let cfg = ImpostorConfig::default();
        assert!(!cfg.enabled);
        assert!(cfg.distance_threshold > 0.0);
        assert!(cfg.resolution > 0);
    }

    #[test]
    fn test_billboard_axes_are_normalized() {
        let quad = billboard_quad([0.0, 0.0, 0.0], 2.0, [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]);
        let p0 = quad.points.get(0);
        let p1 = quad.points.get(1);
        let edge =
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2) + (p1[2] - p0[2]).powi(2)).sqrt();
        assert!((edge - 2.0).abs() < 1e-10);
    }

    #[test]
    fn impostor_generation_honors_config_and_actor_bounds() {
        let actor = Actor::new(PolyData::from_triangles(
            vec![[10.0, 0.0, 0.0], [12.0, 0.0, 0.0], [10.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        ))
        .with_position(1.0, 2.0, 3.0)
        .with_scale(2.0);
        let camera = Camera::default();

        let disabled = ImpostorConfig::default();
        assert!(
            generate_impostor_quads_with_config(&[actor.clone()], &camera, &disabled).is_empty()
        );

        let enabled = ImpostorConfig {
            enabled: true,
            distance_threshold: 1.0,
            resolution: 256,
        };
        let quads = generate_impostor_quads_with_config(&[actor], &camera, &enabled);
        assert_eq!(quads.len(), 1);

        let quad = &quads[0].1;
        let center = quad.points.centroid();
        assert!((center[0] - 23.0).abs() < 1e-10);
        assert!((center[1] - 4.0).abs() < 1e-10);
        assert!((center[2] - 3.0).abs() < 1e-10);
    }
}
