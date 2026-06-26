//! Bus stop shelter geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn bus_stop(width: f64, depth: f64, height: f64, roof_overhang: f64) -> PolyData {
    let width = width.max(0.0);
    let depth = depth.max(0.0);
    let height = height.max(0.0);
    let roof_overhang = roof_overhang.max(0.0);
    let hw = width / 2.0;
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
    // Back wall (glass panel)
    let wt = 0.05;
    ab(
        &mut pts,
        &mut polys,
        -hw,
        depth - wt,
        0.0,
        hw,
        depth,
        height,
    );
    // Side walls
    ab(&mut pts, &mut polys, -hw - wt, 0.0, 0.0, -hw, depth, height);
    ab(&mut pts, &mut polys, hw, 0.0, 0.0, hw + wt, depth, height);
    // Roof
    let rb = pts.len();
    pts.push([-hw - roof_overhang, -roof_overhang, height]);
    pts.push([hw + roof_overhang, -roof_overhang, height]);
    pts.push([hw + roof_overhang, depth + roof_overhang, height + 0.1]);
    pts.push([-hw - roof_overhang, depth + roof_overhang, height + 0.1]);
    polys.push_cell(&[rb as i64, (rb + 1) as i64, (rb + 2) as i64, (rb + 3) as i64]);
    // Bench
    let bh = height * 0.35;
    let bd = depth * 0.3;
    ab(
        &mut pts,
        &mut polys,
        -hw * 0.8,
        depth * 0.2,
        bh - 0.05,
        hw * 0.8,
        depth * 0.2 + bd,
        bh,
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
        let b = bus_stop(3.0, 1.5, 2.5, 0.3);
        assert!(b.polys.num_cells() > 15);
    }
}
