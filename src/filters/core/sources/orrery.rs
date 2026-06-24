//! Orrery (mechanical planetary model with orbits and gears).
use crate::data::{CellArray, Points, PolyData};

pub fn orrery(base_radius: f64, n_planets: usize, na: usize) -> PolyData {
    let np = n_planets.max(3).min(9);
    let na = na.max(16);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Base disk
    let bc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([base_radius * a.cos(), base_radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[
            bc as i64,
            (bc + 1 + j) as i64,
            (bc + 1 + (j + 1) % na) as i64,
        ]);
    }
    // Central post (sun)
    let sun_b = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let sun_t = pts.len();
    pts.push([0.0, 0.0, base_radius * 0.5]);
    lines.push_cell(&[sun_b as i64, sun_t as i64]);
    // Sun sphere indicator
    let sun_sphere = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([
            base_radius * 0.05 * a.cos(),
            base_radius * 0.05 * a.sin(),
            base_radius * 0.5,
        ]);
    }
    for j in 0..na {
        lines.push_cell(&[(sun_sphere + j) as i64, (sun_sphere + (j + 1) % na) as i64]);
    }
    // Planet orbits and arms
    for p in 0..np {
        let orbit_r = base_radius * 0.15 * (p + 1) as f64;
        let planet_z = base_radius * 0.3;
        let angle = 2.0 * std::f64::consts::PI * p as f64 / np as f64 * 1.5; // spread
                                                                             // Orbit circle
        let ob = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([orbit_r * a.cos(), orbit_r * a.sin(), planet_z]);
        }
        for j in 0..na {
            lines.push_cell(&[(ob + j) as i64, (ob + (j + 1) % na) as i64]);
        }
        // Arm from center to planet position
        let arm_b = pts.len();
        pts.push([0.0, 0.0, planet_z]);
        let planet_pos = pts.len();
        pts.push([orbit_r * angle.cos(), orbit_r * angle.sin(), planet_z]);
        lines.push_cell(&[arm_b as i64, planet_pos as i64]);
        // Small circle for planet
        let planet_r = base_radius * 0.02 * (1.0 + 0.5 * (p as f64 / np as f64));
        let pb = pts.len();
        for j in 0..8 {
            let a = 2.0 * std::f64::consts::PI * j as f64 / 8.0;
            pts.push([
                orbit_r * angle.cos() + planet_r * a.cos(),
                orbit_r * angle.sin() + planet_r * a.sin(),
                planet_z,
            ]);
        }
        for j in 0..8 {
            lines.push_cell(&[(pb + j) as i64, (pb + (j + 1) % 8) as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_orrery() {
        let m = orrery(5.0, 6, 24);
        assert!(m.points.len() > 100);
        assert!(m.lines.num_cells() > 50);
    }
}
