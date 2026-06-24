//! Large Ferris wheel with spokes, rim, and gondolas.
use crate::data::{CellArray, Points, PolyData};
pub fn large_ferris_wheel(
    radius: f64,
    num_gondolas: usize,
    spoke_count: usize,
    gondola_size: f64,
    hub_height: f64,
) -> PolyData {
    let ng = num_gondolas.max(4);
    let ns = spoke_count.max(ng);
    let gs = gondola_size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // A-frame support
    let spread = radius * 0.35;
    for &(sx, sy) in &[(-spread, 0.0), (spread, 0.0)] {
        let sb = pts.len();
        pts.push([sx, sy - spread * 0.3, 0.0]);
        pts.push([0.0, 0.0, hub_height]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
        let sb2 = pts.len();
        pts.push([sx, sy + spread * 0.3, 0.0]);
        pts.push([0.0, 0.0, hub_height]);
        lines.push_cell(&[sb2 as i64, (sb2 + 1) as i64]);
    }
    // Hub
    let hub = pts.len();
    pts.push([0.0, 0.0, hub_height]);
    // Spokes and rim
    for i in 0..ns {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ns as f64;
        let rx = radius * a.cos();
        let rz = hub_height + radius * a.sin();
        let si = pts.len();
        pts.push([rx, 0.0, rz]);
        lines.push_cell(&[hub as i64, si as i64]);
    }
    let rim_start = hub + 1;
    for i in 0..ns {
        let j = (i + 1) % ns;
        lines.push_cell(&[(rim_start + i) as i64, (rim_start + j) as i64]);
    }
    // Inner rim
    let ir = radius * 0.85;
    let inner_start = pts.len();
    for i in 0..ns {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ns as f64;
        pts.push([ir * a.cos(), 0.0, hub_height + ir * a.sin()]);
    }
    for i in 0..ns {
        let j = (i + 1) % ns;
        lines.push_cell(&[(inner_start + i) as i64, (inner_start + j) as i64]);
    }
    // Cross bracing between rims
    for i in 0..ns {
        lines.push_cell(&[(rim_start + i) as i64, (inner_start + i) as i64]);
    }
    // Gondolas
    for i in 0..ng {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ng as f64;
        let cx = radius * a.cos();
        let cz = hub_height + radius * a.sin() - gs * 2.5;
        let gb = pts.len();
        pts.push([cx - gs, -gs, cz]);
        pts.push([cx + gs, -gs, cz]);
        pts.push([cx + gs, gs, cz]);
        pts.push([cx - gs, gs, cz]);
        pts.push([cx - gs, -gs, cz + gs * 2.0]);
        pts.push([cx + gs, -gs, cz + gs * 2.0]);
        pts.push([cx + gs, gs, cz + gs * 2.0]);
        pts.push([cx - gs, gs, cz + gs * 2.0]);
        let f = |j: usize| (gb + j) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
        // Hanger
        let hb = pts.len();
        pts.push([cx, 0.0, hub_height + radius * a.sin()]);
        pts.push([cx, 0.0, cz + gs * 2.0]);
        lines.push_cell(&[hb as i64, (hb + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let f = large_ferris_wheel(15.0, 12, 24, 1.5, 18.0);
        assert!(f.polys.num_cells() >= 72);
        assert!(f.lines.num_cells() > 40);
    }
}
