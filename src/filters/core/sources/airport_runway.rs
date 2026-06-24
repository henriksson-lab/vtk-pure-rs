//! Airport runway with markings.
use crate::data::{CellArray, Points, PolyData};
pub fn runway(length: f64, width: f64, marking_count: usize) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main runway surface
    let rb = pts.len();
    pts.push([-hl, -hw, 0.0]);
    pts.push([hl, -hw, 0.0]);
    pts.push([hl, hw, 0.0]);
    pts.push([-hl, hw, 0.0]);
    polys.push_cell(&[rb as i64, (rb + 1) as i64, (rb + 2) as i64, (rb + 3) as i64]);
    // Center line markings
    let mw = width * 0.02;
    let ml = length * 0.04;
    let _gap = length * 0.03;
    let nm = marking_count.max(1);
    for mi in 0..nm {
        let x = -hl * 0.8 + (mi as f64 / (nm as f64)) * (length * 0.8);
        let mb = pts.len();
        pts.push([x, -mw, 0.01]);
        pts.push([x + ml, -mw, 0.01]);
        pts.push([x + ml, mw, 0.01]);
        pts.push([x, mw, 0.01]);
        polys.push_cell(&[mb as i64, (mb + 1) as i64, (mb + 2) as i64, (mb + 3) as i64]);
    }
    // Threshold markings (wide stripes at ends)
    let tw = width * 0.06;
    let tl = length * 0.02;
    for side in [-1.0f64, 1.0] {
        let x = side * (hl - tl * 2.0);
        for si in 0..4 {
            let y = -hw * 0.6 + si as f64 * width * 0.3;
            let tb = pts.len();
            pts.push([x, y, 0.01]);
            pts.push([x + tl * side, y, 0.01]);
            pts.push([x + tl * side, y + tw, 0.01]);
            pts.push([x, y + tw, 0.01]);
            polys.push_cell(&[tb as i64, (tb + 1) as i64, (tb + 2) as i64, (tb + 3) as i64]);
        }
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
        let r = runway(1000.0, 30.0, 10);
        assert!(r.polys.num_cells() > 10);
    }
}
