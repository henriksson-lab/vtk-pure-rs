//! Church building with steeple.
use crate::data::{CellArray, Points, PolyData};
pub fn church(
    width: f64,
    depth: f64,
    wall_h: f64,
    roof_h: f64,
    steeple_h: f64,
    steeple_w: f64,
) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let sw = steeple_w / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main building walls
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]); //0-3
    pts.push([-hw, -hd, wall_h]);
    pts.push([hw, -hd, wall_h]);
    pts.push([hw, hd, wall_h]);
    pts.push([-hw, hd, wall_h]); //4-7
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    polys.push_cell(&[0, 3, 2, 1]); // floor
                                    // Gable roof
    pts.push([0.0, -hd, wall_h + roof_h]); //8
    pts.push([0.0, hd, wall_h + roof_h]); //9
    polys.push_cell(&[4, 8, 9, 7]);
    polys.push_cell(&[5, 6, 9, 8]);
    polys.push_cell(&[4, 5, 8]);
    polys.push_cell(&[6, 7, 9]);
    // Steeple (at front)
    let sz = wall_h;
    pts.push([-sw, -hd, sz]);
    pts.push([sw, -hd, sz]);
    pts.push([sw, -hd + steeple_w, sz]);
    pts.push([-sw, -hd + steeple_w, sz]); //10-13
    pts.push([-sw, -hd, sz + steeple_h * 0.6]);
    pts.push([sw, -hd, sz + steeple_h * 0.6]);
    pts.push([sw, -hd + steeple_w, sz + steeple_h * 0.6]);
    pts.push([-sw, -hd + steeple_w, sz + steeple_h * 0.6]); //14-17
    polys.push_cell(&[10, 11, 15, 14]);
    polys.push_cell(&[11, 12, 16, 15]);
    polys.push_cell(&[12, 13, 17, 16]);
    polys.push_cell(&[13, 10, 14, 17]);
    // Spire
    pts.push([0.0, -hd + sw, sz + steeple_h]); //18
    polys.push_cell(&[14, 15, 18]);
    polys.push_cell(&[15, 16, 18]);
    polys.push_cell(&[16, 17, 18]);
    polys.push_cell(&[17, 14, 18]);
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
        let c = church(6.0, 12.0, 4.0, 2.0, 8.0, 2.0);
        assert!(c.polys.num_cells() > 15);
    }
}
