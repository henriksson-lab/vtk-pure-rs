//! Spinning wheel (fiber craft wheel with treadle).
use crate::data::{CellArray, Points, PolyData};

pub fn spinning_wheel(wheel_radius: f64, na: usize) -> PolyData {
    let na = na.max(16);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Main drive wheel
    let wb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([
            0.0,
            wheel_radius * a.cos(),
            wheel_radius + wheel_radius * a.sin(),
        ]);
    }
    for j in 0..na {
        lines.push_cell(&[(wb + j) as i64, (wb + (j + 1) % na) as i64]);
    }
    // Spokes
    let hub = pts.len();
    pts.push([0.0, 0.0, wheel_radius]);
    for j in 0..8 {
        let idx = wb + j * na / 8;
        lines.push_cell(&[hub as i64, idx as i64]);
    }
    // Axle supports (two A-frames)
    for &y in &[-wheel_radius * 0.15, wheel_radius * 0.15] {
        let s0 = pts.len();
        pts.push([-wheel_radius * 0.3, y, 0.0]);
        let s1 = pts.len();
        pts.push([wheel_radius * 0.1, y, 0.0]);
        lines.push_cell(&[s0 as i64, hub as i64]);
        lines.push_cell(&[s1 as i64, hub as i64]);
    }
    // Flyer/bobbin assembly (small mechanism at side)
    let fly_x = wheel_radius * 0.8;
    let f0 = pts.len();
    pts.push([fly_x, 0.0, wheel_radius]);
    let f1 = pts.len();
    pts.push([fly_x, 0.0, wheel_radius + wheel_radius * 0.3]);
    lines.push_cell(&[f0 as i64, f1 as i64]);
    // Drive band (line from wheel to flyer)
    lines.push_cell(&[(wb + na / 4) as i64, f0 as i64]);
    // Mother-of-all (horizontal beam)
    let m0 = pts.len();
    pts.push([-wheel_radius * 0.3, 0.0, wheel_radius * 0.7]);
    let m1 = pts.len();
    pts.push([fly_x + wheel_radius * 0.1, 0.0, wheel_radius * 0.7]);
    lines.push_cell(&[m0 as i64, m1 as i64]);
    // Treadle
    let t0 = pts.len();
    pts.push([-wheel_radius * 0.2, 0.0, 0.0]);
    let t1 = pts.len();
    pts.push([wheel_radius * 0.3, 0.0, 0.0]);
    lines.push_cell(&[t0 as i64, t1 as i64]);
    // Footman (connects treadle to wheel crank)
    let crank = pts.len();
    pts.push([0.0, 0.0, wheel_radius * 0.15]);
    lines.push_cell(&[t1 as i64, crank as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spinning_wheel() {
        let m = spinning_wheel(3.0, 20);
        assert!(m.points.len() > 25);
        assert!(m.lines.num_cells() > 20);
    }
}
