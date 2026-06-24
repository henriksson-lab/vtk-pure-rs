//! Barn geometry (rectangular building with gambrel or gable roof).
use crate::data::{CellArray, Points, PolyData};
pub fn barn(width: f64, depth: f64, wall_height: f64, roof_height: f64) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Walls
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]); //0-3 base
    pts.push([-hw, -hd, wall_height]);
    pts.push([hw, -hd, wall_height]);
    pts.push([hw, hd, wall_height]);
    pts.push([-hw, hd, wall_height]); //4-7 top
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    // Floor
    polys.push_cell(&[0, 3, 2, 1]);
    // Gable roof
    let ridge_h = wall_height + roof_height;
    pts.push([0.0, -hd, ridge_h]); //8
    pts.push([0.0, hd, ridge_h]); //9
                                  // Front gable
    polys.push_cell(&[4, 5, 8]);
    // Back gable
    polys.push_cell(&[6, 7, 9]);
    // Left roof slope
    polys.push_cell(&[4, 8, 9, 7]);
    // Right roof slope
    polys.push_cell(&[5, 6, 9, 8]);
    // Door opening (decorative quad on front wall)
    let dw = width * 0.15;
    let dh = wall_height * 0.7;
    let db = pts.len();
    pts.push([-dw, -hd - 0.01, 0.0]);
    pts.push([dw, -hd - 0.01, 0.0]);
    pts.push([dw, -hd - 0.01, dh]);
    pts.push([-dw, -hd - 0.01, dh]);
    polys.push_cell(&[db as i64, (db + 1) as i64, (db + 2) as i64, (db + 3) as i64]);
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
        let b = barn(8.0, 12.0, 4.0, 3.0);
        assert!(b.polys.num_cells() >= 9);
    }
}
