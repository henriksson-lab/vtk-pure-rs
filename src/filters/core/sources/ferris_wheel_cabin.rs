//! Detailed Ferris wheel with enclosed cabins.
use crate::data::{CellArray, Points, PolyData};
pub fn ferris_wheel_detailed(
    radius: f64,
    num_cabins: usize,
    cabin_size: f64,
    spoke_count: usize,
) -> PolyData {
    let nc = num_cabins.max(4);
    let ns = spoke_count.max(nc);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hub_z = radius * 1.2;
    // Support legs (A-frames)
    let spread = radius * 0.4;
    for &s in &[-1.0f64, 1.0] {
        let b = pts.len();
        pts.push([s * spread, -spread, 0.0]);
        pts.push([0.0, 0.0, hub_z]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        let b2 = pts.len();
        pts.push([s * spread, spread, 0.0]);
        pts.push([0.0, 0.0, hub_z]);
        lines.push_cell(&[b2 as i64, (b2 + 1) as i64]);
    }
    // Spokes
    let hub = pts.len();
    pts.push([0.0, 0.0, hub_z]);
    for i in 0..ns {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ns as f64;
        let si = pts.len();
        pts.push([radius * a.cos(), 0.0, hub_z + radius * a.sin()]);
        lines.push_cell(&[hub as i64, si as i64]);
    }
    // Rim
    let rim_start = hub + 1;
    for i in 0..ns {
        let j = (i + 1) % ns;
        lines.push_cell(&[(rim_start + i) as i64, (rim_start + j) as i64]);
    }
    // Cabins
    let cs = cabin_size / 2.0;
    for i in 0..nc {
        let a = 2.0 * std::f64::consts::PI * i as f64 / nc as f64;
        let cx = radius * a.cos();
        let cz = hub_z + radius * a.sin() - cs * 2.5;
        let cb = pts.len();
        pts.push([cx - cs, -cs, cz]);
        pts.push([cx + cs, -cs, cz]);
        pts.push([cx + cs, cs, cz]);
        pts.push([cx - cs, cs, cz]);
        pts.push([cx - cs, -cs, cz + cs * 2.0]);
        pts.push([cx + cs, -cs, cz + cs * 2.0]);
        pts.push([cx + cs, cs, cz + cs * 2.0]);
        pts.push([cx - cs, cs, cz + cs * 2.0]);
        let f = |j: usize| (cb + j) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
        // Hanger line
        let hb = pts.len();
        pts.push([cx, 0.0, hub_z + radius * a.sin()]);
        pts.push([cx, 0.0, cz + cs * 2.0]);
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
        let f = ferris_wheel_detailed(10.0, 6, 1.2, 12);
        assert!(f.polys.num_cells() >= 36);
        assert!(f.lines.num_cells() > 15);
    }
}
