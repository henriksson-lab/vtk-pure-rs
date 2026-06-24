//! Pulled rickshaw (two-wheeled cart with shafts).
use crate::data::{CellArray, Points, PolyData};

pub fn rickshaw(wheel_radius: f64, seat_width: f64, na: usize) -> PolyData {
    let na = na.max(12);
    let hw = seat_width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Two wheels
    for &y in &[-hw * 1.2, hw * 1.2] {
        let wb = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                wheel_radius * a.cos(),
                y,
                wheel_radius + wheel_radius * a.sin(),
            ]);
        }
        for j in 0..na {
            lines.push_cell(&[(wb + j) as i64, (wb + (j + 1) % na) as i64]);
        }
        // Spokes
        let hub = pts.len();
        pts.push([0.0, y, wheel_radius]);
        for j in 0..6 {
            lines.push_cell(&[hub as i64, (wb + j * na / 6) as i64]);
        }
    }
    // Axle
    let ax0 = pts.len();
    pts.push([0.0, -hw * 1.2, wheel_radius]);
    let ax1 = pts.len();
    pts.push([0.0, hw * 1.2, wheel_radius]);
    lines.push_cell(&[ax0 as i64, ax1 as i64]);
    // Seat
    let seat_h = wheel_radius * 1.5;
    let seat_d = wheel_radius * 0.8;
    let sb = pts.len();
    pts.push([-seat_d / 2.0, -hw, seat_h]);
    pts.push([seat_d / 2.0, -hw, seat_h]);
    pts.push([seat_d / 2.0, hw, seat_h]);
    pts.push([-seat_d / 2.0, hw, seat_h]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
    // Backrest
    let br = pts.len();
    pts.push([-seat_d / 2.0, -hw, seat_h]);
    pts.push([-seat_d / 2.0, hw, seat_h]);
    pts.push([-seat_d / 2.0, hw, seat_h + wheel_radius * 0.8]);
    pts.push([-seat_d / 2.0, -hw, seat_h + wheel_radius * 0.8]);
    polys.push_cell(&[br as i64, (br + 1) as i64, (br + 2) as i64, (br + 3) as i64]);
    // Shafts (two long poles extending forward)
    for &y in &[-hw * 0.7, hw * 0.7] {
        let s0 = pts.len();
        pts.push([seat_d / 2.0, y, seat_h]);
        let s1 = pts.len();
        pts.push([seat_d + wheel_radius * 3.0, y, seat_h * 0.7]);
        lines.push_cell(&[s0 as i64, s1 as i64]);
    }
    // Canopy frame
    let cf0 = pts.len();
    pts.push([-seat_d / 2.0, -hw, seat_h + wheel_radius * 1.5]);
    let cf1 = pts.len();
    pts.push([seat_d / 2.0, -hw, seat_h + wheel_radius * 1.5]);
    let cf2 = pts.len();
    pts.push([seat_d / 2.0, hw, seat_h + wheel_radius * 1.5]);
    let cf3 = pts.len();
    pts.push([-seat_d / 2.0, hw, seat_h + wheel_radius * 1.5]);
    lines.push_cell(&[cf0 as i64, cf1 as i64]);
    lines.push_cell(&[cf1 as i64, cf2 as i64]);
    lines.push_cell(&[cf2 as i64, cf3 as i64]);
    lines.push_cell(&[cf3 as i64, cf0 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rickshaw() {
        let m = rickshaw(1.0, 1.2, 16);
        assert!(m.points.len() > 40);
        assert!(m.polys.num_cells() >= 2);
        assert!(m.lines.num_cells() > 20);
    }
}
