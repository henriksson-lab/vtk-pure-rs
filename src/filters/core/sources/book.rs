//! Book-shaped geometry (rectangular prism with spine).
use crate::data::{CellArray, Points, PolyData};
pub fn book(width: f64, height: f64, thickness: f64) -> PolyData {
    let w = width;
    let h = height;
    let t = thickness;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([w, 0.0, 0.0]);
    pts.push([w, h, 0.0]);
    pts.push([0.0, h, 0.0]);
    pts.push([0.0, 0.0, t]);
    pts.push([w, 0.0, t]);
    pts.push([w, h, t]);
    pts.push([0.0, h, t]);
    let faces = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64, f[3] as i64]);
    }
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
        let b = book(0.2, 0.3, 0.03);
        assert_eq!(b.points.len(), 8);
        assert_eq!(b.polys.num_cells(), 6);
    }
}
