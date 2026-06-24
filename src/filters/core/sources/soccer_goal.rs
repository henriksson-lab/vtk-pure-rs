//! Soccer goal geometry (posts + crossbar + net approximation).
use crate::data::{CellArray, Points, PolyData};
pub fn soccer_goal(width: f64, height: f64, depth: f64, _post_radius: f64) -> PolyData {
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Posts and crossbar
    let lb = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    pts.push([-hw, 0.0, height]);
    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
    let rb = pts.len();
    pts.push([hw, 0.0, 0.0]);
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    let cb = pts.len();
    pts.push([-hw, 0.0, height]);
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    // Back supports
    let lb2 = pts.len();
    pts.push([-hw, -depth, 0.0]);
    pts.push([-hw, 0.0, height]);
    lines.push_cell(&[lb2 as i64, (lb2 + 1) as i64]);
    let rb2 = pts.len();
    pts.push([hw, -depth, 0.0]);
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[rb2 as i64, (rb2 + 1) as i64]);
    let bb = pts.len();
    pts.push([-hw, -depth, 0.0]);
    pts.push([hw, -depth, 0.0]);
    lines.push_cell(&[bb as i64, (bb + 1) as i64]);
    // Net panels (simplified as quads)
    // Back net
    let nb = pts.len();
    pts.push([-hw, 0.0, height]);
    pts.push([hw, 0.0, height]);
    pts.push([hw, -depth, 0.0]);
    pts.push([-hw, -depth, 0.0]);
    polys.push_cell(&[nb as i64, (nb + 1) as i64, (nb + 2) as i64, (nb + 3) as i64]);
    // Side nets
    let sb = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    pts.push([-hw, 0.0, height]);
    pts.push([-hw, -depth, 0.0]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64]);
    let sb2 = pts.len();
    pts.push([hw, 0.0, 0.0]);
    pts.push([hw, 0.0, height]);
    pts.push([hw, -depth, 0.0]);
    polys.push_cell(&[sb2 as i64, (sb2 + 2) as i64, (sb2 + 1) as i64]);
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
        let g = soccer_goal(7.32, 2.44, 2.0, 0.06);
        assert!(g.lines.num_cells() >= 6);
        assert!(g.polys.num_cells() >= 3);
    }
}
