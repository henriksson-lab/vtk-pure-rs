//! Coal power plant with cooling towers and smokestacks.
use crate::data::{CellArray, Points, PolyData};
pub fn coal_power_plant(
    building_w: f64,
    building_l: f64,
    building_h: f64,
    tower_r: f64,
    tower_h: f64,
    stack_r: f64,
    stack_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let ab = |pts: &mut Points<f64>,
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
    // Main building
    ab(
        &mut pts, &mut polys, 0.0, 0.0, 0.0, building_l, building_w, building_h,
    );
    // Cooling towers (hyperbolic-ish profile)
    for &(tx, ty) in &[
        (building_l * 1.3, 0.0),
        (building_l * 1.3, building_w * 0.8),
    ] {
        let nseg = 8;
        for is in 0..=nseg {
            let t = is as f64 / nseg as f64;
            let z = t * tower_h;
            let r = tower_r * (1.0 + 0.3 * (std::f64::consts::PI * t).sin() - 0.15 * t);
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([tx + r * a.cos(), ty + r * a.sin(), z]);
            }
        }
        let base = pts.len() - (nseg + 1) * res;
        for is in 0..nseg {
            for i in 0..res {
                let j = (i + 1) % res;
                polys.push_cell(&[
                    (base + is * res + i) as i64,
                    (base + is * res + j) as i64,
                    (base + (is + 1) * res + j) as i64,
                    (base + (is + 1) * res + i) as i64,
                ]);
            }
        }
    }
    // Smokestacks
    for &sx in &[building_l * 0.3, building_l * 0.6] {
        let sb = pts.len();
        for ring in 0..=1 {
            let z = if ring == 0 {
                building_h
            } else {
                building_h + stack_h
            };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([
                    sx + stack_r * a.cos(),
                    building_w * 0.5 + stack_r * a.sin(),
                    z,
                ]);
            }
        }
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (sb + i) as i64,
                (sb + j) as i64,
                (sb + res + j) as i64,
                (sb + res + i) as i64,
            ]);
        }
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
        let p = coal_power_plant(20.0, 40.0, 15.0, 8.0, 30.0, 2.0, 25.0, 8);
        assert!(p.polys.num_cells() > 50);
    }
}
