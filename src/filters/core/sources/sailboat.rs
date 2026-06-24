//! Sailboat geometry (hull + mast + sails).
use crate::data::{CellArray, Points, PolyData};
pub fn sailboat(hull_l: f64, hull_w: f64, hull_h: f64, mast_h: f64) -> PolyData {
    let hl = hull_l / 2.0;
    let hw = hull_w / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Hull (simplified boat shape)
    pts.push([-hl, 0.0, -hull_h]); //0 bow bottom
    pts.push([hl * 0.8, -hw, -hull_h * 0.5]); //1
    pts.push([hl, -hw, 0.0]); //2
    pts.push([hl, hw, 0.0]); //3
    pts.push([hl * 0.8, hw, -hull_h * 0.5]); //4
    pts.push([-hl * 0.5, -hw, 0.0]); //5
    pts.push([-hl * 0.5, hw, 0.0]); //6
                                    // Hull faces
    polys.push_cell(&[0, 1, 2, 5]); // port side
    polys.push_cell(&[0, 6, 3, 4]); // starboard
    polys.push_cell(&[0, 5, 6]); // bow
    polys.push_cell(&[2, 3, 4, 1]); // stern
    polys.push_cell(&[5, 2, 3, 6]); // deck
                                    // Mast
    let mb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, mast_h]);
    lines.push_cell(&[mb as i64, (mb + 1) as i64]);
    // Main sail (triangle)
    let sb = pts.len();
    pts.push([0.0, 0.0, mast_h * 0.1]);
    pts.push([0.0, 0.0, mast_h * 0.95]);
    pts.push([hull_l * 0.35, 0.0, mast_h * 0.1]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64]);
    // Jib sail
    let jb = pts.len();
    pts.push([0.0, 0.0, mast_h * 0.8]);
    pts.push([-hl * 0.8, 0.0, mast_h * 0.05]);
    pts.push([0.0, 0.0, mast_h * 0.05]);
    polys.push_cell(&[jb as i64, (jb + 1) as i64, (jb + 2) as i64]);
    // Boom
    let bb = pts.len();
    pts.push([0.0, 0.0, mast_h * 0.1]);
    pts.push([hull_l * 0.35, 0.0, mast_h * 0.1]);
    lines.push_cell(&[bb as i64, (bb + 1) as i64]);
    // Keel
    let kb = pts.len();
    pts.push([0.0, 0.0, -hull_h]);
    pts.push([0.0, 0.0, -hull_h * 2.5]);
    lines.push_cell(&[kb as i64, (kb + 1) as i64]);
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
        let s = sailboat(8.0, 2.5, 1.0, 10.0);
        assert!(s.polys.num_cells() >= 7);
        assert!(s.lines.num_cells() >= 3);
    }
}
