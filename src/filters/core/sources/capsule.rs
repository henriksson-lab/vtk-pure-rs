use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a capsule (cylinder with hemispherical caps).
pub struct CapsuleParams {
    /// Radius of the capsule. Default: 0.5
    pub radius: f64,
    /// Length of the cylindrical section (total height = length + 2*radius). Default: 1.0
    pub length: f64,
    /// Number of sides around the circumference. Default: 32
    pub theta_resolution: usize,
    /// Number of rings per hemisphere. Default: 8
    pub phi_resolution: usize,
    /// Center of the capsule. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for CapsuleParams {
    fn default() -> Self {
        Self {
            radius: 0.5,
            length: 1.0,
            theta_resolution: 32,
            phi_resolution: 8,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a capsule (cylinder + two hemisphere caps) aligned along the Y axis.
pub fn capsule(params: &CapsuleParams) -> PolyData {
    let n_theta = params.theta_resolution.max(3);
    let n_phi = params.phi_resolution.max(2);
    let r = params.radius;
    let half_len = params.length * 0.5;
    let cx = params.center[0];
    let cy = params.center[1];
    let cz = params.center[2];

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Top pole
    points.push([cx, cy + half_len + r, cz]);
    normals.push_tuple(&[0.0, 1.0, 0.0]);

    // Top hemisphere rings (from pole down to equator)
    for j in 1..n_phi {
        let phi = PI * 0.5 * j as f64 / n_phi as f64;
        let sp = phi.sin();
        let cp = phi.cos();
        for i in 0..n_theta {
            let theta = 2.0 * PI * i as f64 / n_theta as f64;
            let x = cx + r * sp * theta.cos();
            let y = cy + half_len + r * cp;
            let z = cz + r * sp * theta.sin();
            points.push([x, y, z]);
            normals.push_tuple(&[sp * theta.cos(), cp, sp * theta.sin()]);
        }
    }

    // Cylinder top ring (equator of top hemisphere)
    let cyl_top_start = points.len();
    for i in 0..n_theta {
        let theta = 2.0 * PI * i as f64 / n_theta as f64;
        let x = cx + r * theta.cos();
        let z = cz + r * theta.sin();
        points.push([x, cy + half_len, z]);
        normals.push_tuple(&[theta.cos(), 0.0, theta.sin()]);
    }

    // Cylinder bottom ring
    let cyl_bot_start = points.len();
    for i in 0..n_theta {
        let theta = 2.0 * PI * i as f64 / n_theta as f64;
        let x = cx + r * theta.cos();
        let z = cz + r * theta.sin();
        points.push([x, cy - half_len, z]);
        normals.push_tuple(&[theta.cos(), 0.0, theta.sin()]);
    }

    // Bottom hemisphere rings (from equator down to pole)
    let bot_hemi_start = points.len();
    for j in 1..n_phi {
        let phi = PI * 0.5 * j as f64 / n_phi as f64;
        let sp = phi.sin();
        let cp = phi.cos();
        for i in 0..n_theta {
            let theta = 2.0 * PI * i as f64 / n_theta as f64;
            let x = cx + r * cp * theta.cos();
            let y = cy - half_len - r * sp;
            let z = cz + r * cp * theta.sin();
            points.push([x, y, z]);
            normals.push_tuple(&[cp * theta.cos(), -sp, cp * theta.sin()]);
        }
    }

    // Bottom pole
    let bottom_pole = points.len() as i64;
    points.push([cx, cy - half_len - r, cz]);
    normals.push_tuple(&[0.0, -1.0, 0.0]);

    // --- Triangulate ---

    // Top cap (pole to first ring)
    for i in 0..n_theta {
        let i_next = (i + 1) % n_theta;
        polys.push_cell(&[0, (1 + i) as i64, (1 + i_next) as i64]);
    }

    // Top hemisphere body
    for j in 0..n_phi - 2 {
        for i in 0..n_theta {
            let i_next = (i + 1) % n_theta;
            let a = (1 + j * n_theta + i) as i64;
            let b = (1 + j * n_theta + i_next) as i64;
            let c = (1 + (j + 1) * n_theta + i_next) as i64;
            let d = (1 + (j + 1) * n_theta + i) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
    }

    // Top hemisphere last ring to cylinder top
    if n_phi >= 2 {
        let last_ring = 1 + (n_phi - 2) * n_theta;
        for i in 0..n_theta {
            let i_next = (i + 1) % n_theta;
            let a = (last_ring + i) as i64;
            let b = (last_ring + i_next) as i64;
            let c = (cyl_top_start + i_next) as i64;
            let d = (cyl_top_start + i) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
    }

    // Cylinder body
    for i in 0..n_theta {
        let i_next = (i + 1) % n_theta;
        let a = (cyl_top_start + i) as i64;
        let b = (cyl_top_start + i_next) as i64;
        let c = (cyl_bot_start + i_next) as i64;
        let d = (cyl_bot_start + i) as i64;
        polys.push_cell(&[a, b, c, d]);
    }

    // Cylinder bottom to bottom hemisphere first ring
    for i in 0..n_theta {
        let i_next = (i + 1) % n_theta;
        let a = (cyl_bot_start + i) as i64;
        let b = (cyl_bot_start + i_next) as i64;
        let c = (bot_hemi_start + i_next) as i64;
        let d = (bot_hemi_start + i) as i64;
        polys.push_cell(&[a, b, c, d]);
    }

    // Bottom hemisphere body
    for j in 0..n_phi - 2 {
        for i in 0..n_theta {
            let i_next = (i + 1) % n_theta;
            let a = (bot_hemi_start + j * n_theta + i) as i64;
            let b = (bot_hemi_start + j * n_theta + i_next) as i64;
            let c = (bot_hemi_start + (j + 1) * n_theta + i_next) as i64;
            let d = (bot_hemi_start + (j + 1) * n_theta + i) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
    }

    // Bottom cap (last ring to bottom pole)
    let last_bot = bot_hemi_start + (n_phi - 2) * n_theta;
    for i in 0..n_theta {
        let i_next = (i + 1) % n_theta;
        polys.push_cell(&[
            (last_bot + i) as i64,
            bottom_pole,
            (last_bot + i_next) as i64,
        ]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(AnyDataArray::F64(normals));
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_capsule() {
        let pd = capsule(&CapsuleParams::default());
        assert!(pd.points.len() > 100);
        assert!(pd.polys.num_cells() > 50);
        assert!(pd.point_data().get_array("Normals").is_some());
    }

    #[test]
    fn small_capsule() {
        let pd = capsule(&CapsuleParams {
            theta_resolution: 4,
            phi_resolution: 2,
            ..Default::default()
        });
        // 2 poles + 1 top hemi ring + 2 cyl rings + 1 bot hemi ring = 2 + 4*4 = 18
        assert_eq!(pd.points.len(), 18);
    }

    #[test]
    fn top_bottom_extremes() {
        let params = CapsuleParams {
            radius: 1.0,
            length: 2.0,
            theta_resolution: 4,
            phi_resolution: 2,
            center: [0.0, 0.0, 0.0],
            ..Default::default()
        };
        let pd = capsule(&params);
        // Top pole at y = length/2 + radius = 2.0
        let top = pd.points.get(0);
        assert!((top[1] - 2.0).abs() < 1e-10);
        // Bottom pole
        let bot = pd.points.get(pd.points.len() - 1);
        assert!((bot[1] + 2.0).abs() < 1e-10);
    }
}
