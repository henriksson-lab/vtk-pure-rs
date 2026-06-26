//! Grandfather clock (tall case with pendulum and face).
use crate::data::{CellArray, Points, PolyData};

pub fn grandfather_clock(height: f64, width: f64, depth: f64) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Main case (tall box)
    let cb = pts.len();
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    pts.push([-hw, -hd, height]);
    pts.push([hw, -hd, height]);
    pts.push([hw, hd, height]);
    pts.push([-hw, hd, height]);
    polys.push_cell(&[cb as i64, (cb + 1) as i64, (cb + 5) as i64, (cb + 4) as i64]); // front
    polys.push_cell(&[
        (cb + 1) as i64,
        (cb + 2) as i64,
        (cb + 6) as i64,
        (cb + 5) as i64,
    ]); // right
    polys.push_cell(&[
        (cb + 2) as i64,
        (cb + 3) as i64,
        (cb + 7) as i64,
        (cb + 6) as i64,
    ]); // back
    polys.push_cell(&[(cb + 3) as i64, cb as i64, (cb + 4) as i64, (cb + 7) as i64]); // left
    polys.push_cell(&[
        (cb + 4) as i64,
        (cb + 5) as i64,
        (cb + 6) as i64,
        (cb + 7) as i64,
    ]); // top
    polys.push_cell(&[cb as i64, (cb + 3) as i64, (cb + 2) as i64, (cb + 1) as i64]);
    let crown = pts.len();
    pts.push([0.0, -hd, height + height * 0.08]);
    polys.push_cell(&[(cb + 4) as i64, (cb + 5) as i64, crown as i64]);
    // Clock face circle
    let face_z = height * 0.8;
    let face_r = width * 0.35;
    let na = 20;
    let fb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([face_r * a.cos(), -hd - 0.01, face_z + face_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(fb + j) as i64, (fb + (j + 1) % na) as i64]);
    }
    // Hour markers
    for h in 0..12 {
        let a = 2.0 * std::f64::consts::PI * h as f64 / 12.0;
        let m0 = pts.len();
        pts.push([
            face_r * 0.85 * a.cos(),
            -hd - 0.01,
            face_z + face_r * 0.85 * a.sin(),
        ]);
        let m1 = pts.len();
        pts.push([
            face_r * 0.95 * a.cos(),
            -hd - 0.01,
            face_z + face_r * 0.95 * a.sin(),
        ]);
        lines.push_cell(&[m0 as i64, m1 as i64]);
    }
    // Hands
    let hc = pts.len();
    pts.push([0.0, -hd - 0.01, face_z]);
    let hh = pts.len();
    pts.push([0.0, -hd - 0.01, face_z + face_r * 0.45]);
    let mh = pts.len();
    pts.push([face_r * 0.55, -hd - 0.01, face_z + face_r * 0.15]);
    lines.push_cell(&[hc as i64, hh as i64]);
    lines.push_cell(&[hc as i64, mh as i64]);
    // Pendulum
    let pv = pts.len();
    pts.push([0.0, -hd - 0.01, height * 0.45]);
    let pb = pts.len();
    pts.push([width * 0.1, -hd - 0.01, height * 0.15]);
    lines.push_cell(&[pv as i64, pb as i64]);
    // Pendulum bob circle
    let bob_r = width * 0.06;
    let pbb = pts.len();
    for j in 0..8 {
        let a = 2.0 * std::f64::consts::PI * j as f64 / 8.0;
        pts.push([
            width * 0.1 + bob_r * a.cos(),
            -hd - 0.01,
            height * 0.15 + bob_r * a.sin(),
        ]);
    }
    for j in 0..8 {
        lines.push_cell(&[(pbb + j) as i64, (pbb + (j + 1) % 8) as i64]);
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
    fn test_clock() {
        let m = grandfather_clock(2.0, 0.5, 0.3);
        assert!(m.points.len() > 40);
        assert!(m.polys.num_cells() >= 7);
        assert!(m.lines.num_cells() > 20);
    }
}
