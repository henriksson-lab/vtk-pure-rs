//! Water tower geometry (tank on stilts).
use crate::data::{CellArray, Points, PolyData};
pub fn water_tower(
    tank_radius: f64,
    tank_height: f64,
    leg_height: f64,
    num_legs: usize,
    _leg_radius: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nl = num_legs.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Tank (cylinder at height)
    for ring in 0..=1 {
        let z = leg_height + if ring == 0 { 0.0 } else { tank_height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([tank_radius * a.cos(), tank_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Top cap
    let tc = pts.len();
    pts.push([0.0, 0.0, leg_height + tank_height]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[tc as i64, (res + i) as i64, (res + j) as i64]);
    }
    // Bottom cap
    let bc = pts.len();
    pts.push([0.0, 0.0, leg_height]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[bc as i64, j as i64, i as i64]);
    }
    // Legs
    for li in 0..nl {
        let a = 2.0 * std::f64::consts::PI * li as f64 / nl as f64;
        let x = tank_radius * 0.8 * a.cos();
        let y = tank_radius * 0.8 * a.sin();
        let b = pts.len();
        pts.push([x, y, 0.0]);
        pts.push([x, y, leg_height]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    // Cross bracing
    for li in 0..nl {
        let li2 = (li + 1) % nl;
        let a1 = 2.0 * std::f64::consts::PI * li as f64 / nl as f64;
        let a2 = 2.0 * std::f64::consts::PI * li2 as f64 / nl as f64;
        let b = pts.len();
        pts.push([
            tank_radius * 0.8 * a1.cos(),
            tank_radius * 0.8 * a1.sin(),
            leg_height * 0.5,
        ]);
        pts.push([
            tank_radius * 0.8 * a2.cos(),
            tank_radius * 0.8 * a2.sin(),
            leg_height * 0.5,
        ]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
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
        let w = water_tower(3.0, 4.0, 10.0, 4, 0.3, 12);
        assert!(w.polys.num_cells() > 10);
        assert!(w.lines.num_cells() >= 4);
    }
}
