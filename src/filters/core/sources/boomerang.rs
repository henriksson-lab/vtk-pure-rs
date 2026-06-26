//! Boomerang (V-shaped airfoil).
use crate::data::{CellArray, Points, PolyData};

pub fn boomerang(arm_length: f64, arm_width: f64, bend_angle_deg: f64, n_pts: usize) -> PolyData {
    let np = n_pts.max(10);
    let half_angle = bend_angle_deg * std::f64::consts::PI / 360.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for s in -(np as isize)..=(np as isize) {
        let side = if s < 0 { -1.0 } else { 1.0 };
        let t = s.unsigned_abs() as f64 / np as f64;
        let angle = side * half_angle;
        let dx = angle.sin();
        let dy = angle.cos();
        let nx = -dy;
        let ny = dx;
        let r = arm_length * t;
        let w = arm_width * (1.0 - 0.7 * t);
        let camber = 0.01 * arm_length * (std::f64::consts::PI * t).sin();
        let cx = r * dx;
        let cy = r * dy;
        pts.push([cx + w / 2.0 * nx, cy + w / 2.0 * ny, camber]);
        pts.push([cx - w / 2.0 * nx, cy - w / 2.0 * ny, -camber]);
    }

    for i in 0..(2 * np) {
        let b = i * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
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
        assert_eq!(m.points.len(), 62);
        assert_eq!(m.polys.num_cells(), 30);
    }
}
