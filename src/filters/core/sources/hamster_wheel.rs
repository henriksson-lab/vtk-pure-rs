//! Hamster/exercise wheel with rungs.
use crate::data::{CellArray, Points, PolyData};

pub fn hamster_wheel(radius: f64, width: f64, n_rungs: usize, na: usize) -> PolyData {
    let nr = n_rungs.max(8);
    let na = na.max(nr);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Two rim circles
    for &y in &[-hw, hw] {
        let rb = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([radius * a.cos(), y, radius * a.sin()]);
        }
        for j in 0..na {
            lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
        }
    }
    // Rungs connecting rims
    for r in 0..nr {
        let a = 2.0 * std::f64::consts::PI * r as f64 / nr as f64;
        let rung_r = radius * 0.95;
        let l = pts.len();
        pts.push([rung_r * a.cos(), -hw, rung_r * a.sin()]);
        let r_pt = pts.len();
        pts.push([rung_r * a.cos(), hw, rung_r * a.sin()]);
        lines.push_cell(&[l as i64, r_pt as i64]);
    }
    // Axle
    let ax0 = pts.len();
    pts.push([0.0, -hw * 1.5, 0.0]);
    let ax1 = pts.len();
    pts.push([0.0, hw * 1.5, 0.0]);
    lines.push_cell(&[ax0 as i64, ax1 as i64]);
    // Spokes from axle to rim
    for &y in &[-hw, hw] {
        for s in 0..4 {
            let a = std::f64::consts::PI / 2.0 * s as f64;
            let hub = pts.len();
            pts.push([0.0, y, 0.0]);
            let rim = pts.len();
            pts.push([radius * a.cos(), y, radius * a.sin()]);
            lines.push_cell(&[hub as i64, rim as i64]);
        }
    }
    // Stand
    let stand_h = radius * 1.3;
    for &y in &[-hw * 1.3, hw * 1.3] {
        let sb = pts.len();
        pts.push([0.0, y, 0.0]);
        let sf = pts.len();
        pts.push([radius * 0.5, y, -stand_h]);
        let sr = pts.len();
        pts.push([-radius * 0.5, y, -stand_h]);
        lines.push_cell(&[sb as i64, sf as i64]);
        lines.push_cell(&[sb as i64, sr as i64]);
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
    fn test_hamster() {
        let m = hamster_wheel(3.0, 1.5, 12, 24);
        assert!(m.points.len() > 60);
        assert!(m.lines.num_cells() > 40);
    }
}
