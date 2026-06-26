//! Spiral staircase geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn spiral_staircase(
    inner_radius: f64,
    outer_radius: f64,
    height: f64,
    steps: usize,
    turns: f64,
) -> PolyData {
    let steps = steps.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let angle_per_step = 2.0 * std::f64::consts::PI * turns / steps as f64;
    let height_per_step = height / steps as f64;
    for i in 0..steps {
        let a0 = i as f64 * angle_per_step;
        let a1 = (i + 1) as f64 * angle_per_step;
        let z = i as f64 * height_per_step;
        let b = pts.len();
        pts.push([inner_radius * a0.cos(), inner_radius * a0.sin(), z]);
        pts.push([outer_radius * a0.cos(), outer_radius * a0.sin(), z]);
        pts.push([outer_radius * a1.cos(), outer_radius * a1.sin(), z]);
        pts.push([inner_radius * a1.cos(), inner_radius * a1.sin(), z]);
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
    }
    // Central pole
    let bc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, height]);
    let mut lines = CellArray::new();
    lines.push_cell(&[bc as i64, (bc + 1) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let s = spiral_staircase(0.3, 1.5, 5.0, 20, 2.0);
        assert_eq!(s.polys.num_cells(), 20);
    }
}
