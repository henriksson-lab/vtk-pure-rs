//! Ballista (giant crossbow siege weapon).
use crate::data::{CellArray, Points, PolyData};

pub fn ballista(frame_length: f64, arm_span: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Main frame (rectangular base)
    let fl = frame_length;
    let fw = fl * 0.3;
    let f0 = pts.len();
    pts.push([-fl / 2.0, -fw / 2.0, 0.0]);
    let f1 = pts.len();
    pts.push([fl / 2.0, -fw / 2.0, 0.0]);
    let f2 = pts.len();
    pts.push([fl / 2.0, fw / 2.0, 0.0]);
    let f3 = pts.len();
    pts.push([-fl / 2.0, fw / 2.0, 0.0]);
    lines.push_cell(&[f0 as i64, f1 as i64]);
    lines.push_cell(&[f1 as i64, f2 as i64]);
    lines.push_cell(&[f2 as i64, f3 as i64]);
    lines.push_cell(&[f3 as i64, f0 as i64]);
    // Uprights
    let h = fl * 0.4;
    let u0 = pts.len();
    pts.push([-fl * 0.1, -fw / 2.0, h]);
    let u1 = pts.len();
    pts.push([-fl * 0.1, fw / 2.0, h]);
    lines.push_cell(&[f0 as i64, u0 as i64]);
    lines.push_cell(&[f3 as i64, u1 as i64]);
    lines.push_cell(&[u0 as i64, u1 as i64]); // crossbar
                                              // Bow arms (from crossbar outward and curved)
    let na = 6;
    for &side in &[-1.0f64, 1.0] {
        let arm_base = pts.len();
        for i in 0..=na {
            let t = i as f64 / na as f64;
            let y = side * (fw / 2.0 + arm_span / 2.0 * t);
            let x = -fl * 0.1 - arm_span * 0.1 * (std::f64::consts::PI * t).sin();
            let z = h - arm_span * 0.05 * t;
            pts.push([x, y, z]);
        }
        for i in 0..na {
            lines.push_cell(&[(arm_base + i) as i64, (arm_base + i + 1) as i64]);
        }
    }
    // Bowstring
    let _left_tip = pts.len() - na - 1 + na; // approximate
    let _right_tip = pts.len() - 1;
    // Track/slide
    let s0 = pts.len();
    pts.push([-fl * 0.1, 0.0, h * 0.8]);
    let s1 = pts.len();
    pts.push([fl * 0.4, 0.0, h * 0.8]);
    lines.push_cell(&[s0 as i64, s1 as i64]);
    // Bolt
    let b0 = pts.len();
    pts.push([0.0, 0.0, h * 0.8 + 0.01]);
    let b1 = pts.len();
    pts.push([fl * 0.5, 0.0, h * 0.8 + 0.01]);
    lines.push_cell(&[b0 as i64, b1 as i64]);
    // Wheels
    let wheel_r = fl * 0.08;
    for &(wx, wy) in &[
        (-fl * 0.35, -fw / 2.0),
        (fl * 0.35, -fw / 2.0),
        (-fl * 0.35, fw / 2.0),
        (fl * 0.35, fw / 2.0),
    ] {
        let wb = pts.len();
        for j in 0..8 {
            let a = 2.0 * std::f64::consts::PI * j as f64 / 8.0;
            pts.push([wx + wheel_r * a.cos(), wy, wheel_r + wheel_r * a.sin()]);
        }
        for j in 0..8 {
            lines.push_cell(&[(wb + j) as i64, (wb + (j + 1) % 8) as i64]);
        }
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
    fn test_ballista() {
        let m = ballista(5.0, 3.0);
        assert!(m.points.len() > 40);
        assert!(m.lines.num_cells() > 20);
    }
}
