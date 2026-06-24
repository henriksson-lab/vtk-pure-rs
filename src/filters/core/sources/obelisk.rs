//! Obelisk (Egyptian monument) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn obelisk(base_size: f64, top_size: f64, height: f64, pyramidion_h: f64) -> PolyData {
    let hb = base_size / 2.0;
    let ht = top_size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base
    pts.push([-hb, -hb, 0.0]);
    pts.push([hb, -hb, 0.0]);
    pts.push([hb, hb, 0.0]);
    pts.push([-hb, hb, 0.0]);
    // Top of shaft
    pts.push([-ht, -ht, height]);
    pts.push([ht, -ht, height]);
    pts.push([ht, ht, height]);
    pts.push([-ht, ht, height]);
    // Pyramidion apex
    pts.push([0.0, 0.0, height + pyramidion_h]);
    // Shaft faces
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    // Base
    polys.push_cell(&[0, 3, 2, 1]);
    // Pyramidion
    polys.push_cell(&[4, 5, 8]);
    polys.push_cell(&[5, 6, 8]);
    polys.push_cell(&[6, 7, 8]);
    polys.push_cell(&[7, 4, 8]);
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
        let o = obelisk(2.0, 1.0, 10.0, 1.5);
        assert_eq!(o.polys.num_cells(), 9);
        assert_eq!(o.points.len(), 9);
    }
}
