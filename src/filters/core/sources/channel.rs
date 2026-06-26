//! U-channel (C-channel) structural shape.
use crate::data::{CellArray, Points, PolyData};
pub fn u_channel(width: f64, height: f64, thickness: f64, depth: f64) -> PolyData {
    let w = width;
    let h = height;
    let t = thickness;
    let hd = depth / 2.0;
    let z0 = -hd;
    let z1 = hd;
    let profile = [
        [0.0, 0.0],
        [w, 0.0],
        [w, t],
        [t, t],
        [t, h - t],
        [w, h - t],
        [w, h],
        [0.0, h],
    ];
    let np = profile.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for p in &profile {
        pts.push([p[0], p[1], z0]);
    }
    for p in &profile {
        pts.push([p[0], p[1], z1]);
    }

    // Front cap: lower flange, web, upper flange.
    polys.push_cell(&[0, 1, 2, 3]);
    polys.push_cell(&[0, 3, 4, 7]);
    polys.push_cell(&[7, 4, 5, 6]);

    // Back cap, opposite winding.
    polys.push_cell(&[np as i64, (np + 3) as i64, (np + 2) as i64, (np + 1) as i64]);
    polys.push_cell(&[np as i64, (np + 7) as i64, (np + 4) as i64, (np + 3) as i64]);
    polys.push_cell(&[
        (np + 7) as i64,
        (np + 6) as i64,
        (np + 5) as i64,
        (np + 4) as i64,
    ]);

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
        assert_eq!(c.points.len(), 16);
        assert_eq!(c.polys.num_cells(), 14);
    }
}
