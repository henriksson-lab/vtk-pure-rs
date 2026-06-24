//! U-channel (C-channel) structural shape.
use crate::data::{CellArray, Points, PolyData};
pub fn u_channel(width: f64, height: f64, thickness: f64, depth: f64) -> PolyData {
    let w = width;
    let h = height;
    let t = thickness;
    let hd = depth / 2.0;
    let profile = [
        [0.0, 0.0],
        [w, 0.0],
        [w, t],
        [t, t],
        [t, h - t],
        [w, h - t],
        [w, h],
        [0.0, h],
        [0.0, h - t],
        [w - t, h - t],
        [w - t, t],
        [0.0, t],
    ];
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
        let c = u_channel(2.0, 3.0, 0.2, 0.5);
        assert_eq!(c.points.len(), 24);
        assert!(c.polys.num_cells() > 10);
    }
}
