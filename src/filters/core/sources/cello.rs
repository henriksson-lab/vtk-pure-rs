//! Cello body outline with strings and bridge.
use crate::data::{CellArray, Points, PolyData};

pub fn cello(body_length: f64, n_pts: usize) -> PolyData {
    let n = n_pts.max(20);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Body outline (two symmetric curves)
    for &side in &[-1.0f64, 1.0] {
        let sb = pts.len();
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let z = body_length * t;
            let w = body_length
                * 0.15
                * (1.0 + 0.4 * (2.0 * std::f64::consts::PI * t).cos()
                    - 0.2 * (4.0 * std::f64::consts::PI * t).cos());
            pts.push([side * w, 0.0, z]);
        }
        for i in 0..n {
            lines.push_cell(&[(sb + i) as i64, (sb + i + 1) as i64]);
        }
    }
    // Connect top and bottom
    lines.push_cell(&[0, (n + 1) as i64]); // bottom
    lines.push_cell(&[n as i64, (2 * n + 1) as i64]); // top
                                                      // Strings (4)
    let string_spread = body_length * 0.04;
    for s in 0..4 {
        let x = -string_spread * 1.5 + string_spread * s as f64;
        let s0 = pts.len();
        pts.push([x, -0.01, body_length * 0.15]);
        let s1 = pts.len();
        pts.push([x, -0.01, body_length * 0.85]);
        lines.push_cell(&[s0 as i64, s1 as i64]);
    }
    // Bridge
    let br0 = pts.len();
    pts.push([-string_spread * 2.0, -0.01, body_length * 0.45]);
    let br1 = pts.len();
    pts.push([string_spread * 2.0, -0.01, body_length * 0.45]);
    lines.push_cell(&[br0 as i64, br1 as i64]);
    // F-holes
    for &sx in &[-1.0f64, 1.0] {
        let fh0 = pts.len();
        pts.push([sx * body_length * 0.06, -0.01, body_length * 0.38]);
        let fh1 = pts.len();
        pts.push([sx * body_length * 0.06, -0.01, body_length * 0.52]);
        lines.push_cell(&[fh0 as i64, fh1 as i64]);
    }
    // Neck
    let n0 = pts.len();
    pts.push([0.0, -0.01, body_length * 0.85]);
    let n1 = pts.len();
    pts.push([0.0, -0.01, body_length * 1.3]);
    lines.push_cell(&[n0 as i64, n1 as i64]);
    // Endpin
    let ep = pts.len();
    pts.push([0.0, 0.0, -body_length * 0.1]);
    lines.push_cell(&[0, ep as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cello() {
        let m = cello(8.0, 30);
        assert!(m.points.len() > 50);
        assert!(m.lines.num_cells() > 40);
    }
}
