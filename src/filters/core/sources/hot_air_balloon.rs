//! Hot air balloon geometry (envelope + basket + ropes).
use crate::data::{CellArray, Points, PolyData};
pub fn hot_air_balloon(
    envelope_r: f64,
    envelope_h: f64,
    basket_size: f64,
    rope_count: usize,
    resolution: usize,
) -> PolyData {
    let envelope_r = envelope_r.abs();
    let envelope_h = envelope_h.abs();
    let basket_size = basket_size.abs();
    let res = resolution.max(8);
    let vres = res;
    let nr = rope_count.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Envelope (teardrop/onion shape)
    for iv in 0..=vres {
        let t = iv as f64 / vres as f64;
        let r = envelope_r * (std::f64::consts::PI * t).sin() * (1.0 - t * 0.3);
        let z = envelope_h * t;
        for iu in 0..res {
            let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for iv in 0..vres {
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (iv * res + iu) as i64,
                (iv * res + iu1) as i64,
                ((iv + 1) * res + iu1) as i64,
                ((iv + 1) * res + iu) as i64,
            ]);
        }
    }
    // Basket (box below)
    let bs = basket_size / 2.0;
    let bz = -envelope_h * 0.15;
    let bb = pts.len();
    pts.push([-bs, -bs, bz - bs]);
    pts.push([bs, -bs, bz - bs]);
    pts.push([bs, bs, bz - bs]);
    pts.push([-bs, bs, bz - bs]);
    pts.push([-bs, -bs, bz]);
    pts.push([bs, -bs, bz]);
    pts.push([bs, bs, bz]);
    pts.push([-bs, bs, bz]);
    let f = |i: usize| (bb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(3), f(0), f(4), f(7)]);
    // Ropes connecting basket to envelope base
    for ri in 0..nr {
        let a = 2.0 * std::f64::consts::PI * ri as f64 / nr as f64;
        let env_r = envelope_r * 0.9;
        let rb = pts.len();
        pts.push([bs * 0.8 * a.cos(), bs * 0.8 * a.sin(), bz]);
        pts.push([
            env_r * a.cos() * 0.3,
            env_r * a.sin() * 0.3,
            envelope_h * 0.05,
        ]);
        lines.push_cell(&[rb as i64, (rb + 1) as i64]);
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
        let b = hot_air_balloon(4.0, 8.0, 1.5, 8, 12);
        assert!(b.polys.num_cells() > 80);
        assert!(b.lines.num_cells() >= 8);
    }
}
