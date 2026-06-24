//! Roller skate (boot with 4 wheels).
use crate::data::{CellArray, Points, PolyData};

pub fn roller_skate(boot_length: f64, boot_height: f64, wheel_radius: f64, na: usize) -> PolyData {
    let na = na.max(10);
    let hw = boot_length * 0.15;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Boot (simplified box)
    let bb = pts.len();
    pts.push([0.0, -hw, wheel_radius * 2.0]);
    pts.push([boot_length, -hw, wheel_radius * 2.0]);
    pts.push([boot_length, hw, wheel_radius * 2.0]);
    pts.push([0.0, hw, wheel_radius * 2.0]);
    pts.push([0.0, -hw, wheel_radius * 2.0 + boot_height]);
    pts.push([boot_length * 0.7, -hw, wheel_radius * 2.0 + boot_height]);
    pts.push([boot_length * 0.7, hw, wheel_radius * 2.0 + boot_height]);
    pts.push([0.0, hw, wheel_radius * 2.0 + boot_height]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 5) as i64, (bb + 4) as i64]);
    polys.push_cell(&[
        (bb + 1) as i64,
        (bb + 2) as i64,
        (bb + 6) as i64,
        (bb + 5) as i64,
    ]);
    polys.push_cell(&[
        (bb + 2) as i64,
        (bb + 3) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
    ]);
    polys.push_cell(&[(bb + 3) as i64, bb as i64, (bb + 4) as i64, (bb + 7) as i64]);
    // Sole plate
    let sb = pts.len();
    pts.push([0.0, -hw * 0.8, wheel_radius * 2.0]);
    pts.push([boot_length, -hw * 0.8, wheel_radius * 2.0]);
    pts.push([boot_length, hw * 0.8, wheel_radius * 2.0]);
    pts.push([0.0, hw * 0.8, wheel_radius * 2.0]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
    // 4 wheels (circles on each side)
    let wheel_positions = [
        boot_length * 0.15,
        boot_length * 0.4,
        boot_length * 0.65,
        boot_length * 0.9,
    ];
    for &wx in &wheel_positions {
        for &side in &[-hw * 0.9, hw * 0.9] {
            let wb = pts.len();
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([
                    wx + wheel_radius * a.cos(),
                    side,
                    wheel_radius + wheel_radius * a.sin(),
                ]);
            }
            for j in 0..na {
                lines.push_cell(&[(wb + j) as i64, (wb + (j + 1) % na) as i64]);
            }
        }
    }
    // Axles connecting wheel pairs
    for &wx in &wheel_positions {
        let a0 = pts.len();
        pts.push([wx, -hw * 0.9, wheel_radius]);
        let a1 = pts.len();
        pts.push([wx, hw * 0.9, wheel_radius]);
        lines.push_cell(&[a0 as i64, a1 as i64]);
    }
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
    fn test_skate() {
        let m = roller_skate(3.0, 1.5, 0.3, 10);
        assert!(m.points.len() > 60);
        assert!(m.polys.num_cells() >= 5);
        assert!(m.lines.num_cells() > 40);
    }
}
