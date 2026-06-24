//! Candelabra (multi-armed candlestick).
use crate::data::{CellArray, Points, PolyData};

pub fn candelabra(height: f64, n_arms: usize) -> PolyData {
    let na = n_arms.max(3);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Central stem
    let s0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let s1 = pts.len();
    pts.push([0.0, 0.0, height]);
    lines.push_cell(&[s0 as i64, s1 as i64]);
    // Base (tripod)
    for i in 0..3 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0;
        let foot = pts.len();
        pts.push([height * 0.2 * a.cos(), height * 0.2 * a.sin(), 0.0]);
        lines.push_cell(&[s0 as i64, foot as i64]);
    }
    // Arms (curved outward and up)
    let arm_z = height * 0.6;
    for i in 0..na {
        let a = 2.0 * std::f64::consts::PI * i as f64 / na as f64;
        let mid = pts.len();
        pts.push([height * 0.15 * a.cos(), height * 0.15 * a.sin(), arm_z]);
        lines.push_cell(&[s1 as i64, mid as i64]); // from center near top
        let tip = pts.len();
        pts.push([
            height * 0.3 * a.cos(),
            height * 0.3 * a.sin(),
            arm_z + height * 0.15,
        ]);
        lines.push_cell(&[mid as i64, tip as i64]);
        // Candle cup (small circle)
        let cup_r = height * 0.03;
        let cb = pts.len();
        for j in 0..6 {
            let ca = 2.0 * std::f64::consts::PI * j as f64 / 6.0;
            pts.push([
                height * 0.3 * a.cos() + cup_r * ca.cos(),
                height * 0.3 * a.sin() + cup_r * ca.sin(),
                arm_z + height * 0.15,
            ]);
        }
        for j in 0..6 {
            lines.push_cell(&[(cb + j) as i64, (cb + (j + 1) % 6) as i64]);
        }
    }
    // Central candle cup
    let cc_r = height * 0.04;
    let ccb = pts.len();
    for j in 0..6 {
        let a = 2.0 * std::f64::consts::PI * j as f64 / 6.0;
        pts.push([cc_r * a.cos(), cc_r * a.sin(), height]);
    }
    for j in 0..6 {
        lines.push_cell(&[(ccb + j) as i64, (ccb + (j + 1) % 6) as i64]);
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
    fn test_candelabra() {
        let m = candelabra(5.0, 5);
        assert!(m.points.len() > 30);
        assert!(m.lines.num_cells() > 20);
    }
}
