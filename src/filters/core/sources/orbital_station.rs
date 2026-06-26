//! Modular orbital space station with multiple module types.
use crate::data::{CellArray, Points, PolyData};
pub fn orbital_station(
    hub_r: f64,
    spoke_count: usize,
    spoke_l: f64,
    module_r: f64,
    module_l: f64,
    ring_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let ns = spoke_count.max(2);
    let ring_radius = if ring_r > 0.0 { ring_r } else { spoke_l };
    let module_steps = ((module_l.abs() / module_r.abs().max(1e-15)).ceil() as usize).max(1);
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
        pts.push([0.0, ring_radius * a.cos(), ring_radius * a.sin()]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
    }
    // Habitat ring
    let ring_steps = ns * res * module_steps;
    let ring_base = pts.len();
    for ai in 0..ring_steps {
        let a = 2.0 * std::f64::consts::PI * ai as f64 / ring_steps as f64;
        for ri in 0..res {
            let ra = 2.0 * std::f64::consts::PI * ri as f64 / res as f64;
            let r = ring_radius + module_r * ra.cos();
            pts.push([module_r * ra.sin(), r * a.cos(), r * a.sin()]);
        }
    }
    for ai in 0..ring_steps {
        let ai1 = (ai + 1) % ring_steps;
        for ri in 0..res {
            let ri1 = (ri + 1) % res;
            polys.push_cell(&[
                (ring_base + ai * res + ri) as i64,
                (ring_base + ai * res + ri1) as i64,
                (ring_base + ai1 * res + ri1) as i64,
                (ring_base + ai1 * res + ri) as i64,
            ]);
        }
    }
    // Solar panels
    for si in 0..ns {
        let a = 2.0 * std::f64::consts::PI * si as f64 / ns as f64;
        let pw = spoke_l * 0.4;
        let ph = spoke_l * 0.2;
        let mid_y = ring_radius * 0.5 * a.cos();
        let mid_z = ring_radius * 0.5 * a.sin();
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
