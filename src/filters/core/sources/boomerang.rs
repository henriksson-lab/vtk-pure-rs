//! Boomerang (V-shaped airfoil).
use crate::data::{CellArray, Points, PolyData};

pub fn boomerang(arm_length: f64, arm_width: f64, bend_angle_deg: f64, n_pts: usize) -> PolyData {
    let np = n_pts.max(10);
    let half_angle = bend_angle_deg * std::f64::consts::PI / 360.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Two arms, each a tapered strip
    for &side in &[-1.0f64, 1.0] {
        let angle = side * half_angle;
        let dx = angle.sin();
        let dy = angle.cos();
        let nx = -dy;
        let ny = dx; // perpendicular
        let base = pts.len();
        for i in 0..=np {
            let t = i as f64 / np as f64;
            let r = arm_length * t;
            let w = arm_width * (1.0 - 0.7 * t); // taper
            let cx = r * dx;
            let cy = r * dy;
            pts.push([
                cx + w / 2.0 * nx,
                cy + w / 2.0 * ny,
                0.01 * arm_length * (std::f64::consts::PI * t).sin(),
            ]);
            pts.push([
                cx - w / 2.0 * nx,
                cy - w / 2.0 * ny,
                -0.01 * arm_length * (std::f64::consts::PI * t).sin(),
            ]);
        }
        for i in 0..np {
            let b = base + i * 2;
            polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boomerang() {
        let m = boomerang(3.0, 0.4, 120.0, 15);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 20);
    }
}
