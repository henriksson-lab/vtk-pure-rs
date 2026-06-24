//! Simple catapult with arm and bucket.
use crate::data::{CellArray, Points, PolyData};
pub fn catapult(base_w: f64, arm_length: f64, arm_angle: f64) -> PolyData {
    let hw = base_w / 2.0;
    let ar = arm_angle.to_radians();
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Base
    let bb = pts.len();
    pts.push([-hw, -hw * 0.3, 0.0]);
    pts.push([hw, -hw * 0.3, 0.0]);
    pts.push([hw, hw * 0.3, 0.0]);
    pts.push([-hw, hw * 0.3, 0.0]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // Uprights
    let uh = base_w * 0.6;
    for &y in &[-hw * 0.2, hw * 0.2] {
        let ub = pts.len();
        pts.push([0.0, y, 0.0]);
        pts.push([0.0, y, uh]);
        lines.push_cell(&[ub as i64, (ub + 1) as i64]);
    }
    // Arm (pivots at top of uprights)
    let pivot_z = uh;
    let arm_short = arm_length * 0.25;
    let arm_long = arm_length * 0.75;
    let tip_x = arm_long * ar.cos();
    let tip_z = pivot_z + arm_long * ar.sin();
    let back_x = -arm_short * ar.cos();
    let back_z = pivot_z - arm_short * ar.sin();
    let ab = pts.len();
    pts.push([back_x, 0.0, back_z]);
    pts.push([tip_x, 0.0, tip_z]);
    lines.push_cell(&[ab as i64, (ab + 1) as i64]);
    // Bucket at tip
    let bs = arm_length * 0.08;
    let bkb = pts.len();
    pts.push([tip_x - bs, 0.0, tip_z]);
    pts.push([tip_x + bs, 0.0, tip_z]);
    pts.push([tip_x + bs, 0.0, tip_z - bs * 2.0]);
    pts.push([tip_x - bs, 0.0, tip_z - bs * 2.0]);
    polys.push_cell(&[
        bkb as i64,
        (bkb + 1) as i64,
        (bkb + 2) as i64,
        (bkb + 3) as i64,
    ]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = catapult(2.0, 4.0, 45.0);
        assert!(c.polys.num_cells() >= 2);
        assert!(c.lines.num_cells() >= 3);
    }
}
