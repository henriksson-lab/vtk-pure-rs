//! Conveyor belt (looped ribbon) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn conveyor_belt(length: f64, width: f64, roller_radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let half_l = length / 2.0;
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let _total = res * 2 + 2; // top straight + left arc + bottom straight + right arc
    let mut profile = Vec::new();
    // Top straight
    let top_n = res / 2;
    for i in 0..=top_n {
        let t = i as f64 / top_n as f64;
        profile.push([half_l - length * t, roller_radius]);
    }
    // Left semicircle
    let arc_n = res / 4;
    for i in 1..arc_n {
        let a = std::f64::consts::PI * i as f64 / arc_n as f64;
        profile.push([-half_l - roller_radius * a.sin(), roller_radius * a.cos()]);
    }
    // Bottom straight
    for i in 0..=top_n {
        let t = i as f64 / top_n as f64;
        profile.push([-half_l + length * t, -roller_radius]);
    }
    // Right semicircle
    for i in 1..arc_n {
        let a = std::f64::consts::PI * i as f64 / arc_n as f64;
        profile.push([half_l + roller_radius * a.sin(), -roller_radius * a.cos()]);
    }
    let np = profile.len();
    for p in &profile {
        pts.push([p[0], -hw, p[1]]);
    }
    for p in &profile {
        pts.push([p[0], hw, p[1]]);
    }
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[i as i64, (np + i) as i64, (np + j) as i64, j as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = conveyor_belt(4.0, 1.0, 0.5, 16);
        assert!(c.points.len() > 10);
        assert!(c.polys.num_cells() > 5);
    }
}
