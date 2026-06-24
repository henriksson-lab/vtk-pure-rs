//! Pagoda (multi-tiered tower) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn pagoda(base_size: f64, tiers: usize, tier_height: f64, overhang: f64) -> PolyData {
    let nt = tiers.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let add_box = |pts: &mut Points<f64>,
                   polys: &mut CellArray,
                   x0: f64,
                   y0: f64,
                   z0: f64,
                   x1: f64,
                   y1: f64,
                   z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    for ti in 0..nt {
        let shrink = ti as f64 * base_size * 0.1;
        let hw = base_size / 2.0 - shrink;
        let z = ti as f64 * tier_height;
        // Wall
        add_box(
            &mut pts,
            &mut polys,
            -hw,
            -hw,
            z,
            hw,
            hw,
            z + tier_height * 0.7,
        );
        // Roof overhang
        let rw = hw + overhang - ti as f64 * overhang * 0.15;
        let rz = z + tier_height * 0.7;
        // Roof as pyramid
        let rb = pts.len();
        pts.push([-rw, -rw, rz]);
        pts.push([rw, -rw, rz]);
        pts.push([rw, rw, rz]);
        pts.push([-rw, rw, rz]);
        pts.push([0.0, 0.0, rz + tier_height * 0.3]);
        polys.push_cell(&[rb as i64, (rb + 1) as i64, (rb + 4) as i64]);
        polys.push_cell(&[(rb + 1) as i64, (rb + 2) as i64, (rb + 4) as i64]);
        polys.push_cell(&[(rb + 2) as i64, (rb + 3) as i64, (rb + 4) as i64]);
        polys.push_cell(&[(rb + 3) as i64, rb as i64, (rb + 4) as i64]);
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
        let p = pagoda(6.0, 5, 3.0, 1.5);
        assert!(p.polys.num_cells() > 40);
    }
}
