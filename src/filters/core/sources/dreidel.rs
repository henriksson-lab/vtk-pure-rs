//! Dreidel (four-sided spinning top).
use crate::data::{CellArray, Points, PolyData};

pub fn dreidel(size: f64) -> PolyData {
    let hs = size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Body (cube)
    pts.push([-hs, -hs, 0.0]);
    pts.push([hs, -hs, 0.0]);
    pts.push([hs, hs, 0.0]);
    pts.push([-hs, hs, 0.0]);
    pts.push([-hs, -hs, size]);
    pts.push([hs, -hs, size]);
    pts.push([hs, hs, size]);
    pts.push([-hs, hs, size]);
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    polys.push_cell(&[4, 5, 6, 7]); // top
                                    // Bottom point (pyramid below body)
    let tip = pts.len();
    pts.push([0.0, 0.0, -size * 0.5]);
    polys.push_cell(&[0, 1, tip as i64]);
    polys.push_cell(&[1, 2, tip as i64]);
    polys.push_cell(&[2, 3, tip as i64]);
    polys.push_cell(&[3, 0, tip as i64]);
    // Handle on top
    let mut lines = CellArray::new();
    let hb = pts.len();
    pts.push([0.0, 0.0, size]);
    let ht = pts.len();
    pts.push([0.0, 0.0, size * 1.5]);
    lines.push_cell(&[hb as i64, ht as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dreidel() {
        let m = dreidel(2.0);
        assert_eq!(m.points.len(), 11);
        assert_eq!(m.polys.num_cells(), 9);
    }
}
