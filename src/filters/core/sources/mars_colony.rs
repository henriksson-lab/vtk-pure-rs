//! Mars colony with pressurized habitats and greenhouses.
use crate::data::{CellArray, Points, PolyData};
pub fn mars_colony(
    hab_r: f64,
    num_habs: usize,
    greenhouse_l: f64,
    greenhouse_w: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nh = num_habs.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Habitats (buried cylinders with dome tops)
    for hi in 0..nh {
        let hx = hi as f64 * (hab_r * 3.0);
        // Cylinder
        for ring in 0..=1 {
            let z = if ring == 0 { -hab_r * 0.5 } else { 0.0 };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([hx + hab_r * a.cos(), hab_r * a.sin(), z]);
            }
        }
        let base = pts.len() - res * 2;
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (base + i) as i64,
                (base + j) as i64,
                (base + res + j) as i64,
                (base + res + i) as i64,
            ]);
        }
        // Dome
        let dome_rings = 4;
        let dome_base = base + res;
        for dr in 1..=dome_rings {
            let t = dr as f64 / dome_rings as f64;
            let a = t * std::f64::consts::FRAC_PI_2;
            let r = hab_r * a.cos();
            let z = hab_r * 0.5 * a.sin();
            for i in 0..res {
                let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([hx + r * ang.cos(), r * ang.sin(), z]);
            }
        }
        for dr in 0..dome_rings {
            let r0 = if dr == 0 {
                dome_base
            } else {
                dome_base + res + (dr - 1) * res
            };
            let r1 = dome_base + res + dr * res;
            if dr == 0 {
                for i in 0..res {
                    let j = (i + 1) % res;
                    polys.push_cell(&[
                        (dome_base + i) as i64,
                        (dome_base + j) as i64,
                        (r1 + j) as i64,
                        (r1 + i) as i64,
                    ]);
                }
            } else {
                for i in 0..res {
                    let j = (i + 1) % res;
                    polys.push_cell(&[
                        (r0 + i) as i64,
                        (r0 + j) as i64,
                        (r1 + j) as i64,
                        (r1 + i) as i64,
                    ]);
                }
            }
        }
        let dc = pts.len();
        pts.push([hx, 0.0, hab_r * 0.5]);
        let top = dome_base + res + (dome_rings - 1) * res;
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[dc as i64, (top + i) as i64, (top + j) as i64]);
        }
    }
    // Greenhouse (quonset hut)
    let gh_x = (nh as f64 - 1.0) * (hab_r * 3.0) / 2.0;
    let gh_y = hab_r * 3.0;
    let gh_steps = 8;
    let hw = greenhouse_w / 2.0;
    for si in 0..=gh_steps {
        let t = si as f64 / gh_steps as f64;
        let a = std::f64::consts::PI * t;
        let y = hw * a.cos();
        let z = hw * a.sin();
        pts.push([gh_x - greenhouse_l / 2.0, gh_y + y, z]);
        pts.push([gh_x + greenhouse_l / 2.0, gh_y + y, z]);
    }
    let ghb = pts.len() - (gh_steps + 1) * 2;
    for si in 0..gh_steps {
        let b = ghb + si * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
    }
    // Connecting tunnels
    for hi in 0..nh - 1 {
        let x0 = hi as f64 * (hab_r * 3.0) + hab_r;
        let x1 = (hi + 1) as f64 * (hab_r * 3.0) - hab_r;
        let tb = pts.len();
        let tw = hab_r * 0.15;
        pts.push([x0, -tw, -tw]);
        pts.push([x1, -tw, -tw]);
        pts.push([x1, tw, -tw]);
        pts.push([x0, tw, -tw]);
        pts.push([x0, -tw, tw]);
        pts.push([x1, -tw, tw]);
        pts.push([x1, tw, tw]);
        pts.push([x0, tw, tw]);
        let f = |i: usize| (tb + i) as i64;
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
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
        let c = mars_colony(4.0, 3, 10.0, 6.0, 8);
        assert!(c.polys.num_cells() > 50);
    }
}
