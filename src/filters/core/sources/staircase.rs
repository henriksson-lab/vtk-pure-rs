//! Staircase geometry source.
use crate::data::{CellArray, Points, PolyData};
/// Create a staircase with N steps.
pub fn staircase(steps: usize, step_width: f64, step_height: f64, step_depth: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..steps {
        let x = i as f64 * step_depth;
        let y = i as f64 * step_height;
        let b = pts.len();
        // 8 vertices per step (box)
        pts.push([x, 0.0, y]);
        pts.push([x + step_depth, 0.0, y]);
        pts.push([x + step_depth, step_width, y]);
        pts.push([x, step_width, y]);
        pts.push([x, 0.0, y + step_height]);
        pts.push([x + step_depth, 0.0, y + step_height]);
        pts.push([x + step_depth, step_width, y + step_height]);
        pts.push([x, step_width, y + step_height]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]); // bottom
        polys.push_cell(&[f(4), f(5), f(6), f(7)]); // top
        polys.push_cell(&[f(0), f(1), f(5), f(4)]); // front
        polys.push_cell(&[f(2), f(3), f(7), f(6)]); // back
        polys.push_cell(&[f(0), f(4), f(7), f(3)]); // left
        polys.push_cell(&[f(1), f(2), f(6), f(5)]); // right
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
    fn test_stairs() {
        let s = staircase(5, 1.0, 0.3, 0.5);
        assert_eq!(s.points.len(), 40);
        assert_eq!(s.polys.num_cells(), 30);
    }
}
