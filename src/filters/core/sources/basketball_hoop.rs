//! Basketball hoop with backboard.
use crate::data::{CellArray, Points, PolyData};
pub fn basketball_hoop(
    pole_height: f64,
    rim_radius: f64,
    backboard_w: f64,
    backboard_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Pole
    let pb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, pole_height]);
    lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    // Backboard
    let bhw = backboard_w / 2.0;
    let bhh = backboard_h / 2.0;
    let bz = pole_height;
    let bb = pts.len();
    pts.push([-bhw, 0.0, bz - bhh]);
    pts.push([bhw, 0.0, bz - bhh]);
    pts.push([bhw, 0.0, bz + bhh]);
    pts.push([-bhw, 0.0, bz + bhh]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // Rim (circle)
    let rim_y = rim_radius * 1.5;
    let rim_z = bz - bhh * 0.3;
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        let j = (i + 1) % res;
        let a2 = 2.0 * std::f64::consts::PI * j as f64 / res as f64;
        let rb = pts.len();
        pts.push([rim_radius * a.cos(), rim_y + rim_radius * a.sin(), rim_z]);
        pts.push([rim_radius * a2.cos(), rim_y + rim_radius * a2.sin(), rim_z]);
        lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    }
    // Support arm (pole to rim)
    let ab = pts.len();
    pts.push([0.0, 0.0, rim_z]);
    pts.push([0.0, rim_y, rim_z]);
    lines.push_cell(&[ab as i64, (ab + 1) as i64]);
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
        let h = basketball_hoop(3.05, 0.23, 1.8, 1.05, 12);
        assert!(h.lines.num_cells() > 10);
    }
}
