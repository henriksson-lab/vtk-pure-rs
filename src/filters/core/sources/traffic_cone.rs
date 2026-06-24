//! Traffic cone geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn traffic_cone(base_radius: f64, height: f64, tip_radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base ring
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([base_radius * a.cos(), base_radius * a.sin(), 0.0]);
    }
    // Top ring
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([tip_radius * a.cos(), tip_radius * a.sin(), height]);
    }
    // Cone surface
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Base (square base for stability)
    let sq = base_radius * 1.5;
    let sb = pts.len();
    pts.push([-sq, -sq, 0.0]);
    pts.push([sq, -sq, 0.0]);
    pts.push([sq, sq, 0.0]);
    pts.push([-sq, sq, 0.0]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
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
        let c = traffic_cone(0.3, 0.7, 0.02, 12);
        assert!(c.polys.num_cells() > 10);
    }
}
