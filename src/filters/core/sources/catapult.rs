//! Catapult/trebuchet geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn trebuchet(base_w: f64, base_d: f64, tower_h: f64, arm_length: f64) -> PolyData {
    let hw = base_w / 2.0;
    let hd = base_d / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Base frame
    let b = pts.len();
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
    // A-frame towers (two sides)
    for &y in &[-hd * 0.5, hd * 0.5] {
        let tb = pts.len();
        pts.push([-hw * 0.3, y, 0.0]);
        pts.push([hw * 0.3, y, 0.0]);
        pts.push([0.0, y, tower_h]);
        lines.push_cell(&[tb as i64, (tb + 2) as i64]);
        lines.push_cell(&[(tb + 1) as i64, (tb + 2) as i64]);
    }
    // Axle
    let ab = pts.len();
    pts.push([0.0, -hd * 0.5, tower_h]);
    pts.push([0.0, hd * 0.5, tower_h]);
    lines.push_cell(&[ab as i64, (ab + 1) as i64]);
    // Throwing arm
    let short = arm_length * 0.3;
    let long = arm_length * 0.7;
    let arm_b = pts.len();
    pts.push([-short, 0.0, tower_h]);
    pts.push([long, 0.0, tower_h]);
    lines.push_cell(&[arm_b as i64, (arm_b + 1) as i64]);
    // Counterweight (box at short end)
    let cw = arm_length * 0.1;
    let cwb = pts.len();
    pts.push([-short - cw, 0.0, tower_h - cw * 2.0]);
    pts.push([-short + cw, 0.0, tower_h - cw * 2.0]);
    pts.push([-short + cw, 0.0, tower_h]);
    pts.push([-short - cw, 0.0, tower_h]);
    polys.push_cell(&[
        cwb as i64,
        (cwb + 1) as i64,
        (cwb + 2) as i64,
        (cwb + 3) as i64,
    ]);
    // Sling (lines from arm tip)
    let sb = pts.len();
    pts.push([long, 0.0, tower_h]);
    pts.push([long + arm_length * 0.2, 0.0, tower_h - arm_length * 0.15]);
    lines.push_cell(&[sb as i64, (sb + 1) as i64]);
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
        let t = trebuchet(3.0, 2.0, 4.0, 5.0);
        assert!(t.lines.num_cells() > 5);
        assert!(t.polys.num_cells() >= 2);
    }
}
