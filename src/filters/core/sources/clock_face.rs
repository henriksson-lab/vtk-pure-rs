//! Clock face with hour markers and hands.
use crate::data::{CellArray, Points, PolyData};
pub fn clock_face(radius: f64, hour: usize, minute: usize) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Dial circle
    let res = 60;
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        let j = (i + 1) % res;
        let a2 = 2.0 * std::f64::consts::PI * j as f64 / res as f64;
        let b = pts.len();
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
        pts.push([radius * a2.cos(), radius * a2.sin(), 0.0]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    // Hour markers
    for i in 0..12 {
        let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::FRAC_PI_6 * i as f64;
        let r1 = radius * 0.85;
        let r2 = radius * 0.95;
        let b = pts.len();
        pts.push([r1 * a.cos(), r1 * a.sin(), 0.0]);
        pts.push([r2 * a.cos(), r2 * a.sin(), 0.0]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    // Hour hand
    let h_angle = std::f64::consts::FRAC_PI_2
        - std::f64::consts::FRAC_PI_6 * (hour % 12) as f64
        - std::f64::consts::FRAC_PI_6 * minute as f64 / 60.0;
    let h_len = radius * 0.5;
    let hb = pts.len();
    pts.push([0.0, 0.0, 0.01]);
    pts.push([h_len * h_angle.cos(), h_len * h_angle.sin(), 0.01]);
    lines.push_cell(&[hb as i64, (hb + 1) as i64]);
    // Minute hand
    let m_angle = std::f64::consts::FRAC_PI_2 - std::f64::consts::PI * 2.0 * minute as f64 / 60.0;
    let m_len = radius * 0.75;
    let mb = pts.len();
    pts.push([0.0, 0.0, 0.02]);
    pts.push([m_len * m_angle.cos(), m_len * m_angle.sin(), 0.02]);
    lines.push_cell(&[mb as i64, (mb + 1) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_noon() {
        let c = clock_face(5.0, 12, 0);
        assert!(c.lines.num_cells() > 60);
        let hour_tip = c.points.get(60 * 2 + 12 * 2 + 1);
        let minute_tip = c.points.get(60 * 2 + 12 * 2 + 3);
        assert!((hour_tip[0] - 0.0).abs() < 1e-10);
        assert!(hour_tip[1] > 0.0);
        assert!((minute_tip[0] - 0.0).abs() < 1e-10);
        assert!(minute_tip[1] > hour_tip[1]);
    }
    #[test]
    fn test_3_15() {
        let c = clock_face(5.0, 3, 15);
        assert!(c.lines.num_cells() > 60);
        let minute_tip = c.points.get(60 * 2 + 12 * 2 + 3);
        assert!(minute_tip[0] > 0.0);
        assert!((minute_tip[1] - 0.0).abs() < 1e-10);
    }
}
