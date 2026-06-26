use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a 2D gear (extruded gear profile).
pub struct GearParams {
    /// Number of teeth on the gear. Default: 12
    pub num_teeth: usize,
    /// Inner radius for the center bore. Default: 0.7
    pub inner_radius: f64,
    /// Outer radius (root circle / base of teeth). Default: 1.0
    pub outer_radius: f64,
    /// Height of the teeth above outer_radius. Default: 0.2
    pub tooth_height: f64,
    /// Thickness (extrusion depth) of the gear. Default: 0.2
    pub thickness: f64,
    /// Center of the gear. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for GearParams {
    fn default() -> Self {
        Self {
            num_teeth: 12,
            inner_radius: 0.7,
            outer_radius: 1.0,
            tooth_height: 0.2,
            thickness: 0.2,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a 2D gear (extruded gear profile) as PolyData with normals.
///
/// The gear profile is created on the XY plane, then extruded along Z by
/// `thickness`. Each tooth is a trapezoidal bump on the outer radius, with
/// `inner_radius` forming the center bore when it is smaller than `outer_radius`.
pub fn gear(params: &GearParams) -> PolyData {
    let nt = params.num_teeth.max(3);
    let r_inner = params.inner_radius.max(0.0);
    let r_outer = params.outer_radius;
    let r_tip = r_outer + params.tooth_height;
    let half_t = params.thickness / 2.0;
    let [cx, cy, cz] = params.center;

    // Build the 2D gear profile (closed polygon in XY).
    // For each tooth: 4 points (root, tooth-start, tooth-end, next-root).
    // Each tooth occupies 1/num_teeth of the circle.
    // Tooth takes up ~40% of the angular span, gap takes ~60%.
    let mut profile: Vec<[f64; 2]> = Vec::new();
    let tooth_frac = 0.4;

    for i in 0..nt {
        let base_angle = 2.0 * PI * i as f64 / nt as f64;
        let span = 2.0 * PI / nt as f64;

        // Gap start (at outer radius)
        let a0 = base_angle;
        profile.push([r_outer * a0.cos(), r_outer * a0.sin()]);

        // Tooth start (ramp up)
        let a1 = base_angle + span * (0.5 - tooth_frac / 2.0);
        profile.push([r_outer * a1.cos(), r_outer * a1.sin()]);

        // Tooth tip start
        profile.push([r_tip * a1.cos(), r_tip * a1.sin()]);

        // Tooth tip end
        let a2 = base_angle + span * (0.5 + tooth_frac / 2.0);
        profile.push([r_tip * a2.cos(), r_tip * a2.sin()]);

        // Tooth end (ramp down)
        profile.push([r_outer * a2.cos(), r_outer * a2.sin()]);
    }

    let n_profile = profile.len();
    let has_bore = r_inner > 0.0 && r_inner < r_outer;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Bottom face points (z = -half_t)
    for &[px, py] in &profile {
        points.push([cx + px, cy + py, cz - half_t]);
        normals.push_tuple(&[0.0, 0.0, -1.0]);
    }
    // Top face points (z = +half_t)
    for &[px, py] in &profile {
        points.push([cx + px, cy + py, cz + half_t]);
        normals.push_tuple(&[0.0, 0.0, 1.0]);
    }

    let top_offset = n_profile as i64;

    if has_bore {
        let bottom_inner_offset = points.len() as i64;
        for i in 0..n_profile {
            let angle = 2.0 * PI * i as f64 / n_profile as f64;
            points.push([
                cx + r_inner * angle.cos(),
                cy + r_inner * angle.sin(),
                cz - half_t,
            ]);
            normals.push_tuple(&[0.0, 0.0, -1.0]);
        }

        let top_inner_offset = points.len() as i64;
        for i in 0..n_profile {
            let angle = 2.0 * PI * i as f64 / n_profile as f64;
            points.push([
                cx + r_inner * angle.cos(),
                cy + r_inner * angle.sin(),
                cz + half_t,
            ]);
            normals.push_tuple(&[0.0, 0.0, 1.0]);
        }

        for i in 0..n_profile {
            let next = (i + 1) % n_profile;
            polys.push_cell(&[
                i as i64,
                bottom_inner_offset + i as i64,
                bottom_inner_offset + next as i64,
                next as i64,
            ]);
            polys.push_cell(&[
                top_offset + i as i64,
                top_offset + next as i64,
                top_inner_offset + next as i64,
                top_inner_offset + i as i64,
            ]);
        }

        for i in 0..n_profile {
            let next = (i + 1) % n_profile;
            let angle0 = 2.0 * PI * i as f64 / n_profile as f64;
            let angle1 = 2.0 * PI * next as f64 / n_profile as f64;
            let nx = -((angle0 + angle1) * 0.5).cos();
            let ny = -((angle0 + angle1) * 0.5).sin();
            let si = points.len() as i64;
            points.push([
                cx + r_inner * angle0.cos(),
                cy + r_inner * angle0.sin(),
                cz - half_t,
            ]);
            normals.push_tuple(&[nx, ny, 0.0]);
            points.push([
                cx + r_inner * angle0.cos(),
                cy + r_inner * angle0.sin(),
                cz + half_t,
            ]);
            normals.push_tuple(&[nx, ny, 0.0]);
            points.push([
                cx + r_inner * angle1.cos(),
                cy + r_inner * angle1.sin(),
                cz + half_t,
            ]);
            normals.push_tuple(&[nx, ny, 0.0]);
            points.push([
                cx + r_inner * angle1.cos(),
                cy + r_inner * angle1.sin(),
                cz - half_t,
            ]);
            normals.push_tuple(&[nx, ny, 0.0]);

            polys.push_cell(&[si, si + 1, si + 2]);
            polys.push_cell(&[si, si + 2, si + 3]);
        }
    } else {
        // Bottom face: fan triangulation from center-ish.
        // Add a center point for the bottom face.
        let bot_center = points.len() as i64;
        points.push([cx, cy, cz - half_t]);
        normals.push_tuple(&[0.0, 0.0, -1.0]);

        for i in 0..n_profile {
            let next = (i + 1) % n_profile;
            polys.push_cell(&[bot_center, next as i64, i as i64]);
        }

        // Top face: fan triangulation.
        let top_center = points.len() as i64;
        points.push([cx, cy, cz + half_t]);
        normals.push_tuple(&[0.0, 0.0, 1.0]);

        for i in 0..n_profile {
            let next = (i + 1) % n_profile;
            polys.push_cell(&[top_center, top_offset + i as i64, top_offset + next as i64]);
        }
    }

    // Side faces: quads connecting bottom and top profiles.
    for i in 0..n_profile {
        let next = (i + 1) % n_profile;
        let _b0 = i as i64;
        let _b1 = next as i64;
        let _t0 = top_offset + i as i64;
        let _t1 = top_offset + next as i64;

        // Compute outward normal for this side edge.
        let dx = profile[next][0] - profile[i][0];
        let dy = profile[next][1] - profile[i][1];
        let len = (dx * dx + dy * dy).sqrt().max(1e-12);
        let nx = dy / len;
        let ny = -dx / len;

        // Add 4 new points with side normals for sharp edges.
        let si = points.len() as i64;
        points.push([cx + profile[i][0], cy + profile[i][1], cz - half_t]);
        normals.push_tuple(&[nx, ny, 0.0]);
        points.push([cx + profile[next][0], cy + profile[next][1], cz - half_t]);
        normals.push_tuple(&[nx, ny, 0.0]);
        points.push([cx + profile[next][0], cy + profile[next][1], cz + half_t]);
        normals.push_tuple(&[nx, ny, 0.0]);
        points.push([cx + profile[i][0], cy + profile[i][1], cz + half_t]);
        normals.push_tuple(&[nx, ny, 0.0]);

        polys.push_cell(&[si, si + 1, si + 2]);
        polys.push_cell(&[si, si + 2, si + 3]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gear() {
        let pd = gear(&GearParams::default());
        assert!(pd.points.len() > 100);
        assert!(pd.polys.num_cells() > 50);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn minimal_gear() {
        let pd = gear(&GearParams {
            num_teeth: 3,
            ..Default::default()
        });
        // 3 teeth * 5 profile pts = 15 profile points
        // bottom + top + 2 centers + 4*15 side points
        assert!(pd.points.len() > 0);
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn gear_has_thickness() {
        let pd = gear(&GearParams {
            thickness: 1.0,
            ..Default::default()
        });
        let zs: Vec<f64> = (0..pd.points.len()).map(|i| pd.points.get(i)[2]).collect();
        let zmin = zs.iter().cloned().fold(f64::INFINITY, f64::min);
        let zmax = zs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((zmax - zmin - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gear_respects_inner_radius() {
        let pd = gear(&GearParams {
            inner_radius: 0.4,
            outer_radius: 1.0,
            ..Default::default()
        });
        let min_radius = pd
            .points
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt())
            .fold(f64::INFINITY, f64::min);
        assert!((min_radius - 0.4).abs() < 1e-10);
    }
}
