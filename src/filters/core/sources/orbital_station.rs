//! Modular orbital space station with multiple module types.
use crate::data::{CellArray, Points, PolyData};
pub fn orbital_station(
    hub_r: f64,
    spoke_count: usize,
    spoke_l: f64,
    module_r: f64,
    _module_l: f64,
    ring_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let ns = spoke_count.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Central hub (cylinder)
    for ring in 0..=1 {
        let x = if ring == 0 { -hub_r } else { hub_r };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, hub_r * 0.5 * a.cos(), hub_r * 0.5 * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Spokes (connecting hub to ring)
    for si in 0..ns {
        let a = 2.0 * std::f64::consts::PI * si as f64 / ns as f64;
        let sb = pts.len();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([0.0, ring_r * a.cos(), ring_r * a.sin()]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
    }
    // Habitat ring (torus segment at each spoke end)
    for si in 0..ns {
        let sa = 2.0 * std::f64::consts::PI * si as f64 / ns as f64;
        let sa2 = 2.0 * std::f64::consts::PI * ((si + 1) % ns) as f64 / ns as f64;
        let cx1 = ring_r * sa.cos();
        let cy1 = ring_r * sa.sin();
        let cx2 = ring_r * sa2.cos();
        let cy2 = ring_r * sa2.sin();
        // Arc segment as quad strip
        let arc_steps = res;
        for ai in 0..=arc_steps {
            let t = ai as f64 / arc_steps as f64;
            let cx = cx1 + (cx2 - cx1) * t;
            let cy = cy1 + (cy2 - cy1) * t;
            for ri in 0..res {
                let ra = 2.0 * std::f64::consts::PI * ri as f64 / res as f64;
                let dx = cx * (1.0 + module_r * ra.cos() / ring_r.max(1e-15));
                let dy = cy * (1.0 + module_r * ra.cos() / ring_r.max(1e-15));
                let dz = module_r * ra.sin();
                pts.push([0.0, dx, dy + dz]);
            }
        }
        let base = pts.len() - (arc_steps + 1) * res;
        for ai in 0..arc_steps {
            for ri in 0..res {
                let ri1 = (ri + 1) % res;
                polys.push_cell(&[
                    (base + ai * res + ri) as i64,
                    (base + ai * res + ri1) as i64,
                    (base + (ai + 1) * res + ri1) as i64,
                    (base + (ai + 1) * res + ri) as i64,
                ]);
            }
        }
    }
    // Solar panels
    for si in 0..ns {
        let a = 2.0 * std::f64::consts::PI * si as f64 / ns as f64;
        let pw = spoke_l * 0.4;
        let ph = spoke_l * 0.2;
        let mid_y = ring_r * 0.5 * a.cos();
        let mid_z = ring_r * 0.5 * a.sin();
        let pb = pts.len();
        pts.push([-pw / 2.0, mid_y - ph / 2.0, mid_z]);
        pts.push([pw / 2.0, mid_y - ph / 2.0, mid_z]);
        pts.push([pw / 2.0, mid_y + ph / 2.0, mid_z]);
        pts.push([-pw / 2.0, mid_y + ph / 2.0, mid_z]);
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
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
        let s = orbital_station(2.0, 4, 8.0, 1.0, 3.0, 10.0, 6);
        assert!(s.polys.num_cells() > 20);
    }
}
