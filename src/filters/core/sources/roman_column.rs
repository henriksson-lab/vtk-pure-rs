//! Roman column with capital and base.
use crate::data::{CellArray, Points, PolyData};
pub fn roman_column(
    shaft_radius: f64,
    height: f64,
    base_size: f64,
    capital_size: f64,
    flutes: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(flutes.max(1) * 2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base (wider square pedestal)
    let bh = height * 0.05;
    let bs = base_size / 2.0;
    let bb = pts.len();
    pts.push([-bs, -bs, 0.0]);
    pts.push([bs, -bs, 0.0]);
    pts.push([bs, bs, 0.0]);
    pts.push([-bs, bs, 0.0]);
    pts.push([-bs, -bs, bh]);
    pts.push([bs, -bs, bh]);
    pts.push([bs, bs, bh]);
    pts.push([-bs, bs, bh]);
    let f = |i: usize| (bb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    // Shaft (fluted cylinder with entasis)
    let shaft_h = height * 0.8;
    let nseg = 8;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let z = bh + t * shaft_h;
        let entasis = 1.0 + 0.03 * (std::f64::consts::PI * t).sin(); // slight bulge
        let r = shaft_radius * entasis;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            let flute_mod = if flutes > 0 {
                1.0 - 0.05 * (a * flutes as f64).cos().max(0.0)
            } else {
                1.0
            };
            pts.push([r * flute_mod * a.cos(), r * flute_mod * a.sin(), z]);
        }
    }
    let shaft_base = bb + 8;
    for is in 0..nseg {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (shaft_base + is * res + i) as i64,
                (shaft_base + is * res + j) as i64,
                (shaft_base + (is + 1) * res + j) as i64,
                (shaft_base + (is + 1) * res + i) as i64,
            ]);
        }
    }
    // Capital (wider top)
    let ch = height * 0.08;
    let cr = capital_size / 2.0;
    let cap_z = bh + shaft_h;
    let cb = pts.len();
    pts.push([-cr, -cr, cap_z]);
    pts.push([cr, -cr, cap_z]);
    pts.push([cr, cr, cap_z]);
    pts.push([-cr, cr, cap_z]);
    pts.push([-cr, -cr, cap_z + ch]);
    pts.push([cr, -cr, cap_z + ch]);
    pts.push([cr, cr, cap_z + ch]);
    pts.push([-cr, cr, cap_z + ch]);
    let g = |i: usize| (cb + i) as i64;
    polys.push_cell(&[g(0), g(3), g(2), g(1)]);
    polys.push_cell(&[g(4), g(5), g(6), g(7)]);
    polys.push_cell(&[g(0), g(1), g(5), g(4)]);
    polys.push_cell(&[g(2), g(3), g(7), g(6)]);
    polys.push_cell(&[g(0), g(4), g(7), g(3)]);
    polys.push_cell(&[g(1), g(2), g(6), g(5)]);
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
        let c = roman_column(0.4, 5.0, 1.2, 1.0, 8, 16);
        assert!(c.polys.num_cells() > 50);
    }
}
