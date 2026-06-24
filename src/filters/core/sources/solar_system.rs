//! Solar system model (planets as spheres on orbital rings).
use crate::data::{CellArray, Points, PolyData};
pub fn solar_system(scale: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Planet data: (orbit_radius, planet_radius, name)
    let planets = [
        (0.39, 0.02),
        (0.72, 0.05),
        (1.0, 0.05),
        (1.52, 0.03),
        (5.2, 0.15),
        (9.5, 0.12),
        (19.2, 0.08),
        (30.1, 0.07),
    ];
    // Sun
    let sun_r = 0.2 * scale;
    let sc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([sun_r * a.cos(), sun_r * a.sin(), 0.0]);
    }
    for i in 0..res {
        let j = if i + 1 < res { sc + 2 + i } else { sc + 1 };
        polys.push_cell(&[sc as i64, (sc + 1 + i) as i64, j as i64]);
    }
    // Planets and orbits
    for &(orbit_r, planet_r) in &planets {
        let or = orbit_r * scale;
        let pr = planet_r * scale;
        // Orbit ring
        let mut orbit_ids = Vec::new();
        for i in 0..res * 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res * 2) as f64;
            let idx = pts.len();
            pts.push([or * a.cos(), or * a.sin(), 0.0]);
            orbit_ids.push(idx as i64);
        }
        orbit_ids.push(orbit_ids[0]); // close the ring
        lines.push_cell(&orbit_ids);
        // Planet (small circle at angle 0)
        let pc = pts.len();
        pts.push([or, 0.0, 0.0]);
        for i in 0..res / 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
            pts.push([or + pr * a.cos(), pr * a.sin(), 0.0]);
        }
        for i in 0..res / 2 {
            let j = if i + 1 < res / 2 { pc + 2 + i } else { pc + 1 };
            polys.push_cell(&[pc as i64, (pc + 1 + i) as i64, j as i64]);
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
        let s = solar_system(10.0, 12);
        assert!(s.polys.num_cells() > 20);
        assert!(s.lines.num_cells() >= 8);
    }
}
