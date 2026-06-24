//! Ship's wheel (helm) with spokes and handles.
use crate::data::{CellArray, Points, PolyData};

pub fn ship_wheel(radius: f64, n_spokes: usize, na: usize) -> PolyData {
    let ns = n_spokes.max(6);
    let na = na.max(ns * 2);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Outer rim
    let rb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
    }
    // Inner hub ring
    let hub_r = radius * 0.15;
    let hb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([hub_r * a.cos(), hub_r * a.sin(), 0.0]);
    }
    for j in 0..na {
        lines.push_cell(&[(hb + j) as i64, (hb + (j + 1) % na) as i64]);
    }
    // Spokes from hub to rim
    for s in 0..ns {
        let a = 2.0 * std::f64::consts::PI * s as f64 / ns as f64;
        let hub_pt = pts.len();
        pts.push([hub_r * a.cos(), hub_r * a.sin(), 0.0]);
        let rim_pt = pts.len();
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
        lines.push_cell(&[hub_pt as i64, rim_pt as i64]);
        // Handle (extending beyond rim)
        let handle_r = radius * 1.2;
        let handle = pts.len();
        pts.push([handle_r * a.cos(), handle_r * a.sin(), 0.0]);
        lines.push_cell(&[rim_pt as i64, handle as i64]);
    }
    // Axle (perpendicular to wheel plane)
    let ax0 = pts.len();
    pts.push([0.0, 0.0, -radius * 0.3]);
    let ax1 = pts.len();
    pts.push([0.0, 0.0, radius * 0.1]);
    lines.push_cell(&[ax0 as i64, ax1 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ship_wheel() {
        let m = ship_wheel(2.0, 8, 24);
        assert!(m.points.len() > 50);
        assert!(m.lines.num_cells() > 40);
    }
}
