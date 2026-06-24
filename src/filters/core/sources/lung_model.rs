//! Simplified lung model (two lobes with bronchial tree).
use crate::data::{CellArray, Points, PolyData};
pub fn lung_model(size: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let s = size;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Two lung lobes (ellipsoids)
    for &(lx, sx, sy) in &[(-s * 0.4, s * 0.35, s * 0.5), (s * 0.4, s * 0.3, s * 0.45)] {
        let lb = pts.len();
        let vres = res;
        for iv in 0..=vres {
            let v = std::f64::consts::PI * iv as f64 / vres as f64;
            for iu in 0..=res {
                let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
                pts.push([
                    lx + sx * v.sin() * u.cos(),
                    sy * v.sin() * u.sin(),
                    s * 0.5 * v.cos(),
                ]);
            }
        }
        let w = res + 1;
        for iv in 0..vres {
            for iu in 0..res {
                polys.push_cell(&[
                    (lb + iv * w + iu) as i64,
                    (lb + iv * w + iu + 1) as i64,
                    (lb + (iv + 1) * w + iu + 1) as i64,
                    (lb + (iv + 1) * w + iu) as i64,
                ]);
            }
        }
    }
    // Trachea + bronchi (branching lines)
    let tb = pts.len();
    pts.push([0.0, 0.0, s * 0.7]);
    pts.push([0.0, 0.0, s * 0.3]);
    lines.push_cell(&[tb as i64, (tb + 1) as i64]);
    // Left bronchus
    let lb2 = pts.len();
    pts.push([0.0, 0.0, s * 0.3]);
    pts.push([-s * 0.3, 0.0, s * 0.1]);
    lines.push_cell(&[lb2 as i64, (lb2 + 1) as i64]);
    // Right bronchus
    let rb = pts.len();
    pts.push([0.0, 0.0, s * 0.3]);
    pts.push([s * 0.3, 0.0, s * 0.1]);
    lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    // Secondary branches
    for &(bx, by) in &[
        (-s * 0.3, -s * 0.1),
        (-s * 0.35, s * 0.1),
        (s * 0.3, -s * 0.1),
        (s * 0.35, s * 0.1),
    ] {
        let sb = pts.len();
        let parent_x = if bx < 0.0 { -s * 0.3 } else { s * 0.3 };
        pts.push([parent_x, 0.0, s * 0.1]);
        pts.push([bx, by, -s * 0.1]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let l = lung_model(5.0, 8);
        assert!(l.polys.num_cells() > 50);
        assert!(l.lines.num_cells() > 5);
    }
}
