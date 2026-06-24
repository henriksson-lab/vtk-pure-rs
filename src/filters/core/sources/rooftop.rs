//! Rooftop geometry (gable, hip, flat, shed).
use crate::data::{CellArray, Points, PolyData};
pub fn gable_roof(width: f64, length: f64, height: f64) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([-hw, -hl, 0.0]);
    pts.push([hw, -hl, 0.0]);
    pts.push([hw, hl, 0.0]);
    pts.push([-hw, hl, 0.0]); // base
    pts.push([0.0, -hl, height]);
    pts.push([0.0, hl, height]); // ridge
    polys.push_cell(&[0, 1, 4]); // front gable
    polys.push_cell(&[2, 3, 5]); // back gable
    polys.push_cell(&[0, 4, 5, 3]); // left slope
    polys.push_cell(&[1, 2, 5, 4]); // right slope
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn hip_roof(width: f64, length: f64, height: f64) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let ridge_offset = hl - hw;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([-hw, -hl, 0.0]);
    pts.push([hw, -hl, 0.0]);
    pts.push([hw, hl, 0.0]);
    pts.push([-hw, hl, 0.0]);
    pts.push([0.0, -ridge_offset, height]);
    pts.push([0.0, ridge_offset, height]);
    polys.push_cell(&[0, 1, 4]);
    polys.push_cell(&[2, 3, 5]);
    polys.push_cell(&[0, 4, 5, 3]);
    polys.push_cell(&[1, 2, 5, 4]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn flat_roof(width: f64, length: f64) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([-hw, -hl, 0.0]);
    pts.push([hw, -hl, 0.0]);
    pts.push([hw, hl, 0.0]);
    pts.push([-hw, hl, 0.0]);
    polys.push_cell(&[0, 1, 2, 3]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn shed_roof(width: f64, length: f64, front_height: f64, back_height: f64) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([-hw, -hl, front_height]);
    pts.push([hw, -hl, front_height]);
    pts.push([hw, hl, back_height]);
    pts.push([-hw, hl, back_height]);
    polys.push_cell(&[0, 1, 2, 3]);
    pts.push([-hw, -hl, 0.0]);
    pts.push([hw, -hl, 0.0]);
    pts.push([hw, hl, 0.0]);
    pts.push([-hw, hl, 0.0]);
    polys.push_cell(&[0, 3, 7, 4]); // left wall
    polys.push_cell(&[1, 5, 6, 2]); // right wall
    polys.push_cell(&[0, 4, 5, 1]); // front wall
    polys.push_cell(&[3, 2, 6, 7]); // back wall
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gable() {
        let r = gable_roof(4.0, 6.0, 2.0);
        assert_eq!(r.polys.num_cells(), 4);
    }
    #[test]
    fn test_hip() {
        let r = hip_roof(4.0, 8.0, 2.0);
        assert_eq!(r.polys.num_cells(), 4);
    }
    #[test]
    fn test_flat() {
        let r = flat_roof(4.0, 6.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_shed() {
        let r = shed_roof(4.0, 6.0, 3.0, 2.0);
        assert_eq!(r.polys.num_cells(), 5);
    }
}
