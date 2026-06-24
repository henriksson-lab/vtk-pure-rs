//! Stethoscope (chest piece, tubing, earpieces).
use crate::data::{CellArray, Points, PolyData};

pub fn stethoscope(tube_length: f64, na: usize) -> PolyData {
    let na = na.max(12);
    let chest_r = tube_length * 0.08;
    let tube_r = tube_length * 0.01;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Chest piece (disk)
    let cc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let cb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([chest_r * a.cos(), chest_r * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[cc as i64, (cb + j) as i64, (cb + (j + 1) % na) as i64]);
    }
    // Rim
    let rb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([chest_r * a.cos(), chest_r * a.sin(), tube_r]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (cb + j) as i64,
            (rb + j) as i64,
            (rb + j1) as i64,
            (cb + j1) as i64,
        ]);
    }
    // Tube (curve from chest piece up and splitting into Y)
    let n_tube = 15;
    let tube_base = pts.len();
    for i in 0..=n_tube {
        let t = i as f64 / n_tube as f64;
        let z = tube_length * t;
        let x = tube_length * 0.1 * (std::f64::consts::PI * t * 0.5).sin();
        pts.push([x, 0.0, z]);
    }
    for i in 0..n_tube {
        lines.push_cell(&[(tube_base + i) as i64, (tube_base + i + 1) as i64]);
    }
    // Y-split at top
    let split_pt = tube_base + n_tube;
    let ear_spread = tube_length * 0.1;
    let ear_l = pts.len();
    pts.push([-ear_spread, 0.0, tube_length + tube_length * 0.1]);
    let ear_r = pts.len();
    pts.push([ear_spread, 0.0, tube_length + tube_length * 0.1]);
    lines.push_cell(&[split_pt as i64, ear_l as i64]);
    lines.push_cell(&[split_pt as i64, ear_r as i64]);
    // Earpieces (small circles)
    for &(ex, ey, ez) in &[
        (-ear_spread, 0.0, tube_length + tube_length * 0.1),
        (ear_spread, 0.0, tube_length + tube_length * 0.1),
    ] {
        let eb = pts.len();
        let er = tube_r * 3.0;
        for j in 0..8 {
            let a = 2.0 * std::f64::consts::PI * j as f64 / 8.0;
            pts.push([ex + er * a.cos(), ey, ez + er * a.sin()]);
        }
        for j in 0..8 {
            lines.push_cell(&[(eb + j) as i64, (eb + (j + 1) % 8) as i64]);
        }
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
    fn test_stethoscope() {
        let m = stethoscope(5.0, 16);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 15);
        assert!(m.lines.num_cells() > 15);
    }
}
