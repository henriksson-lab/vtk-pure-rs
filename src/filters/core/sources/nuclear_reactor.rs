//! Nuclear reactor building with containment dome.
use crate::data::{CellArray, Points, PolyData};
pub fn nuclear_reactor(
    containment_r: f64,
    containment_h: f64,
    dome_h: f64,
    aux_w: f64,
    aux_l: f64,
    aux_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Containment cylinder
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { containment_h };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([containment_r * a.cos(), containment_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Containment dome
    let dome_rings = 6;
    for dr in 1..=dome_rings {
        let t = dr as f64 / dome_rings as f64;
        let a = t * std::f64::consts::FRAC_PI_2;
        let r = containment_r * a.cos();
        let z = containment_h + dome_h * a.sin();
        for i in 0..res {
            let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * ang.cos(), r * ang.sin(), z]);
        }
    }
    let base = res;
    for dr in 0..dome_rings {
        let r0 = if dr == 0 {
            base
        } else {
            base + res + (dr - 1) * res
        };
        let r1 = base + res + dr * res;
        if dr == 0 {
            for i in 0..res {
                let j = (i + 1) % res;
                polys.push_cell(&[
                    (base + i) as i64,
                    (base + j) as i64,
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
    // Dome cap
    let dc = pts.len();
    pts.push([0.0, 0.0, containment_h + dome_h]);
    let top_ring = base + res + (dome_rings - 1) * res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[dc as i64, (top_ring + i) as i64, (top_ring + j) as i64]);
    }
    // Auxiliary building
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
    ab(
        &mut pts,
        &mut polys,
        containment_r * 1.1,
        -aux_w / 2.0,
        0.0,
        containment_r * 1.1 + aux_l,
        aux_w / 2.0,
        aux_h,
    );
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
        let r = nuclear_reactor(15.0, 30.0, 10.0, 20.0, 30.0, 12.0, 12);
        assert!(r.polys.num_cells() > 80);
    }
}
