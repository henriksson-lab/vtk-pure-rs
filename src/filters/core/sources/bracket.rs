//! L-bracket and angle bracket geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn l_bracket(width: f64, height: f64, thickness: f64, depth: f64) -> PolyData {
    let w = width.max(0.0);
    let h = height.max(0.0);
    let t = thickness.max(0.0).min(w).min(h);
    let d = depth.max(0.0);
    let hd = d / 2.0;
    let profile = [[0.0, 0.0], [w, 0.0], [w, t], [t, t], [t, h], [0.0, h]];
    let np = profile.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for p in &profile {
        pts.push([p[0], p[1], -hd]);
    }
    for p in &profile {
        pts.push([p[0], p[1], hd]);
    }
    for i in 1..np - 1 {
        polys.push_cell(&[0, (i + 1) as i64, i as i64]);
    }
    for i in 1..np - 1 {
        polys.push_cell(&[np as i64, (np + i) as i64, (np + i + 1) as i64]);
    }
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[i as i64, j as i64, (np + j) as i64, (np + i) as i64]);
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
        let b = l_bracket(2.0, 3.0, 0.3, 0.5);
        assert_eq!(b.points.len(), 12);
        assert!(b.polys.num_cells() > 8);
    }
}
