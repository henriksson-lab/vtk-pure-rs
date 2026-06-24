//! Step pyramid (Mayan/Egyptian) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn step_pyramid(base_size: f64, height: f64, steps: usize) -> PolyData {
    let ns = steps.max(1);
    let step_h = height / ns as f64;
    let step_shrink = base_size / (2.0 * ns as f64);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let add_box = |pts: &mut Points<f64>,
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
    for si in 0..ns {
        let hw = base_size / 2.0 - si as f64 * step_shrink;
        let z = si as f64 * step_h;
        add_box(&mut pts, &mut polys, -hw, -hw, z, hw, hw, z + step_h);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn smooth_pyramid(base_size: f64, height: f64) -> PolyData {
    let hw = base_size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([-hw, -hw, 0.0]);
    pts.push([hw, -hw, 0.0]);
    pts.push([hw, hw, 0.0]);
    pts.push([-hw, hw, 0.0]);
    pts.push([0.0, 0.0, height]);
    polys.push_cell(&[0, 1, 4]);
    polys.push_cell(&[1, 2, 4]);
    polys.push_cell(&[2, 3, 4]);
    polys.push_cell(&[3, 0, 4]);
    polys.push_cell(&[0, 3, 2, 1]); // base
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step() {
        let p = step_pyramid(10.0, 5.0, 5);
        assert_eq!(p.polys.num_cells(), 30);
    }
    #[test]
    fn test_smooth() {
        let p = smooth_pyramid(4.0, 3.0);
        assert_eq!(p.polys.num_cells(), 5);
    }
}
