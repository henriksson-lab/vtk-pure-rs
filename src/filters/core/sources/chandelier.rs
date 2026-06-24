//! Chandelier with arms and hanging crystals.
use crate::data::{CellArray, Points, PolyData};

pub fn chandelier(radius: f64, n_arms: usize, arm_length: f64) -> PolyData {
    let na = n_arms.max(3);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Central stem
    let s0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let s1 = pts.len();
    pts.push([0.0, 0.0, -0.5]);
    lines.push_cell(&[s0 as i64, s1 as i64]);
    // Hub ring
    let hub_z = -0.5;
    let hub_base = pts.len();
    for i in 0..na {
        let a = 2.0 * std::f64::consts::PI * i as f64 / na as f64;
        pts.push([radius * 0.3 * a.cos(), radius * 0.3 * a.sin(), hub_z]);
    }
    for i in 0..na {
        lines.push_cell(&[(hub_base + i) as i64, (hub_base + (i + 1) % na) as i64]);
    }
    // Arms with candle holders
    for i in 0..na {
        let a = 2.0 * std::f64::consts::PI * i as f64 / na as f64;
        let arm_start = hub_base + i;
        // Arm curves outward and slightly up
        let mid = pts.len();
        pts.push([radius * 0.7 * a.cos(), radius * 0.7 * a.sin(), hub_z - 0.2]);
        let tip = pts.len();
        pts.push([arm_length * a.cos(), arm_length * a.sin(), hub_z + 0.1]);
        lines.push_cell(&[arm_start as i64, mid as i64]);
        lines.push_cell(&[mid as i64, tip as i64]);
        // Crystal drop from midpoint
        let crystal = pts.len();
        pts.push([radius * 0.7 * a.cos(), radius * 0.7 * a.sin(), hub_z - 0.6]);
        lines.push_cell(&[mid as i64, crystal as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_chandelier() {
        let m = chandelier(2.0, 6, 3.0);
        assert!(m.points.len() > 15);
        assert!(m.lines.num_cells() > 10);
    }
}
