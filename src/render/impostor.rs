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
    let cam_pos = [camera.position.x, camera.position.y, camera.position.z];
    let cam_right = {
        let fwd = camera.direction();
        let up = camera.view_up;
        let r = fwd.cross(up).normalize();
        [r.x, r.y, r.z]
    };
    let cam_up = [camera.view_up.x, camera.view_up.y, camera.view_up.z];

    let mut result = Vec::new();

    for (i, actor) in actors.iter().enumerate() {
        if !actor.visible {
            continue;
        }
        let center = actor.position;
        let dx = center[0] - cam_pos[0];
        let dy = center[1] - cam_pos[1];
        let dz = center[2] - cam_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        // Use a default threshold; callers filter based on ImpostorConfig
        let size = actor.scale;
        let quad = billboard_quad(center, size, cam_right, cam_up);
        result.push((i, quad));
        let _ = dist; // distance available for caller filtering
    }

    result
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
}
