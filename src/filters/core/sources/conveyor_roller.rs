//! Conveyor with rollers.
use crate::data::{CellArray, Points, PolyData};
pub fn conveyor_with_rollers(
    length: f64,
    width: f64,
    roller_radius: f64,
    num_rollers: usize,
    resolution: usize,
) -> PolyData {
    let nr = num_rollers.max(2);
    let res = resolution.max(6);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let _lines = CellArray::new();
    // Belt (flat ribbon)
    let belt_h = roller_radius * 1.1;
    let bb = pts.len();
    pts.push([0.0, -hw, belt_h]);
    pts.push([length, -hw, belt_h]);
    pts.push([length, hw, belt_h]);
    pts.push([0.0, hw, belt_h]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // Frame rails
    for &y in &[-hw - roller_radius * 0.5, hw + roller_radius * 0.5] {
        let fb = pts.len();
        pts.push([0.0, y, 0.0]);
        pts.push([length, y, 0.0]);
        pts.push([length, y, belt_h]);
        pts.push([0.0, y, belt_h]);
        polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64, (fb + 3) as i64]);
    }
    // Rollers
    let spacing = length / (nr - 1) as f64;
    for ri in 0..nr {
        let x = ri as f64 * spacing;
        for ring in 0..=1 {
            let y = if ring == 0 { -hw } else { hw };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([
                    x + roller_radius * a.sin(),
                    y,
                    roller_radius + roller_radius * a.cos(),
                ]);
            }
        }
        let rb = pts.len() - res * 2;
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (rb + i) as i64,
                (rb + j) as i64,
                (rb + res + j) as i64,
                (rb + res + i) as i64,
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
        let c = conveyor_with_rollers(6.0, 1.0, 0.15, 5, 8);
        assert!(c.polys.num_cells() > 10);
    }
}
