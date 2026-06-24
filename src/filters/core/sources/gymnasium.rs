//! Gymnasium/sports hall geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn gymnasium(width: f64, length: f64, wall_h: f64, roof_h: f64) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
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
    ab(&mut pts, &mut polys, -hw, -hl, 0.0, hw, hl, wall_h);
    // Barrel vault roof
    let rb = pts.len();
    let res = 12;
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let a = std::f64::consts::PI * t;
        let x = -hw + width * t;
        let z = wall_h + roof_h * a.sin();
        pts.push([x, -hl, z]);
        pts.push([x, hl, z]);
    }
    for i in 0..res {
        let b = rb + i * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
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
        let g = gymnasium(20.0, 30.0, 6.0, 4.0);
        assert!(g.polys.num_cells() > 15);
    }
}
