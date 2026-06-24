//! Modern wind turbine with aerodynamic blades.
use crate::data::{CellArray, Points, PolyData};
pub fn modern_wind_turbine(
    tower_height: f64,
    blade_length: f64,
    num_blades: usize,
    tower_base_r: f64,
    tower_top_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nb = num_blades.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Tapered tower
    let nsec = 6;
    for is in 0..=nsec {
        let t = is as f64 / nsec as f64;
        let r = tower_base_r * (1.0 - t) + tower_top_r * t;
        let z = t * tower_height;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for is in 0..nsec {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (is * res + i) as i64,
                (is * res + j) as i64,
                ((is + 1) * res + j) as i64,
                ((is + 1) * res + i) as i64,
            ]);
        }
    }
    // Nacelle (elongated box)
    let nw = tower_top_r * 2.0;
    let nd = blade_length * 0.12;
    let nh = tower_top_r * 1.5;
    let nb_pts = pts.len();
    pts.push([-nw, -nh / 2.0, tower_height]);
    pts.push([nd, -nh / 2.0, tower_height]);
    pts.push([nd, nh / 2.0, tower_height]);
    pts.push([-nw, nh / 2.0, tower_height]);
    pts.push([-nw, -nh / 2.0, tower_height + nh]);
    pts.push([nd, -nh / 2.0, tower_height + nh]);
    pts.push([nd, nh / 2.0, tower_height + nh]);
    pts.push([-nw, nh / 2.0, tower_height + nh]);
    let f = |i: usize| (nb_pts + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    // Blades (tapered quads in YZ plane at nacelle front)
    let hub_x = nd;
    let hub_z = tower_height + nh / 2.0;
    for bi in 0..nb {
        let angle = 2.0 * std::f64::consts::PI * bi as f64 / nb as f64;
        let blade_steps = 8;
        let base_w = blade_length * 0.05;
        for si in 0..blade_steps {
            let t0 = si as f64 / blade_steps as f64;
            let t1 = (si + 1) as f64 / blade_steps as f64;
            let r0 = blade_length * t0;
            let r1 = blade_length * t1;
            let w0 = base_w * (1.0 - t0 * 0.7);
            let w1 = base_w * (1.0 - t1 * 0.7);
            let ca = angle.cos();
            let sa = angle.sin();
            let bb = pts.len();
            pts.push([hub_x, r0 * ca - w0 * sa, hub_z + r0 * sa + w0 * ca]);
            pts.push([hub_x, r0 * ca + w0 * sa, hub_z + r0 * sa - w0 * ca]);
            pts.push([hub_x, r1 * ca + w1 * sa, hub_z + r1 * sa - w1 * ca]);
            pts.push([hub_x, r1 * ca - w1 * sa, hub_z + r1 * sa + w1 * ca]);
            polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
        }
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
        let w = modern_wind_turbine(50.0, 25.0, 3, 2.5, 1.5, 12);
        assert!(w.polys.num_cells() > 50);
    }
}
