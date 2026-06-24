//! Gear wheel (spur gear) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a spur gear profile extruded along Z.
pub fn gear_wheel(
    num_teeth: usize,
    outer_radius: f64,
    inner_radius: f64,
    tooth_height: f64,
    thickness: f64,
    resolution: usize,
) -> PolyData {
    let teeth = num_teeth.max(3);
    let res_per_tooth = resolution.max(2);
    let _total_pts = teeth * res_per_tooth * 2;
    let half_h = thickness / 2.0;

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Generate gear profile
    let profile: Vec<[f64; 2]> = (0..teeth * res_per_tooth)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / (teeth * res_per_tooth) as f64;
            let tooth_phase = (i % res_per_tooth) as f64 / res_per_tooth as f64;
            let r = if tooth_phase < 0.25 {
                inner_radius + (outer_radius + tooth_height - inner_radius) * (tooth_phase / 0.25)
            } else if tooth_phase < 0.5 {
                outer_radius + tooth_height
            } else if tooth_phase < 0.75 {
                outer_radius + tooth_height
                    - (outer_radius + tooth_height - inner_radius) * ((tooth_phase - 0.5) / 0.25)
            } else {
                inner_radius
            };
            [r * angle.cos(), r * angle.sin()]
        })
        .collect();

    let np = profile.len();
    // Bottom face
    for p in &profile {
        pts.push([p[0], p[1], -half_h]);
    }
    // Top face
    for p in &profile {
        pts.push([p[0], p[1], half_h]);
    }

    // Bottom face (fan from center)
    let bot_center = pts.len();
    pts.push([0.0, 0.0, -half_h]);
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[bot_center as i64, j as i64, i as i64]);
    }

    // Top face
    let top_center = pts.len();
    pts.push([0.0, 0.0, half_h]);
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[top_center as i64, (np + i) as i64, (np + j) as i64]);
    }

    // Side faces
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[i as i64, j as i64, (np + j) as i64, (np + i) as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gear() {
        let g = gear_wheel(12, 2.0, 1.5, 0.3, 0.5, 4);
        assert!(g.points.len() > 50);
        assert!(g.polys.num_cells() > 50);
    }
    #[test]
    fn test_small_gear() {
        let g = gear_wheel(6, 1.0, 0.7, 0.2, 0.3, 3);
        assert!(g.polys.num_cells() > 0);
    }
}
