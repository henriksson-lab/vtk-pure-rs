//! Pendulum clock (case outline with pendulum and hands).
use crate::data::{CellArray, Points, PolyData};

pub fn pendulum_clock(height: f64, width: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let hw = width / 2.0;
    // Case outline
    let c0 = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    let c1 = pts.len();
    pts.push([hw, 0.0, 0.0]);
    let c2 = pts.len();
    pts.push([hw, 0.0, height]);
    let c3 = pts.len();
    pts.push([-hw, 0.0, height]);
    lines.push_cell(&[c0 as i64, c1 as i64]);
    lines.push_cell(&[c1 as i64, c2 as i64]);
    lines.push_cell(&[c2 as i64, c3 as i64]);
    lines.push_cell(&[c3 as i64, c0 as i64]);
    // Clock face circle
    let face_z = height * 0.75;
    let face_r = width * 0.3;
    let na = 24;
    let fb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([face_r * a.cos(), 0.01, face_z + face_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(fb + j) as i64, (fb + (j + 1) % na) as i64]);
    }
    // Hour hand
    let hc = pts.len();
    pts.push([0.0, 0.01, face_z]);
    let hh = pts.len();
    pts.push([0.0, 0.01, face_z + face_r * 0.5]);
    lines.push_cell(&[hc as i64, hh as i64]);
    // Minute hand
    let mh = pts.len();
    pts.push([face_r * 0.7, 0.01, face_z]);
    lines.push_cell(&[hc as i64, mh as i64]);
    // Pendulum
    let pivot = pts.len();
    pts.push([0.0, 0.01, height * 0.45]);
    let bob = pts.len();
    pts.push([width * 0.15, 0.01, height * 0.15]);
    lines.push_cell(&[pivot as i64, bob as i64]);
    // Bob circle
    let bob_r = width * 0.08;
    let bb = pts.len();
    for j in 0..12 {
        let a = 2.0 * std::f64::consts::PI * j as f64 / 12.0;
        pts.push([
            width * 0.15 + bob_r * a.cos(),
            0.01,
            height * 0.15 + bob_r * a.sin(),
        ]);
    }
    for j in 0..12 {
        lines.push_cell(&[(bb + j) as i64, (bb + (j + 1) % 12) as i64]);
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
    fn test_clock() {
        let m = pendulum_clock(10.0, 4.0);
        assert!(m.points.len() > 30);
        assert!(m.lines.num_cells() > 20);
    }
}
