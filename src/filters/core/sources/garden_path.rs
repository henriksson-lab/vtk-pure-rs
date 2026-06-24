//! Garden path (curved walkway) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn curved_path(control_points: &[[f64; 2]], width: f64, resolution: usize) -> PolyData {
    let ncp = control_points.len();
    if ncp < 2 {
        return PolyData::new();
    }
    let res = resolution.max(ncp * 4);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Interpolate path with Catmull-Rom-like
    let total_segs = (ncp - 1) * res / ncp;
    for i in 0..=total_segs {
        let t = i as f64 / total_segs as f64 * (ncp - 1) as f64;
        let seg = t.floor() as usize;
        let frac = t - seg as f64;
        let seg = seg.min(ncp - 2);
        let p0 = control_points[seg];
        let p1 = control_points[seg + 1];
        let x = p0[0] + (p1[0] - p0[0]) * frac;
        let y = p0[1] + (p1[1] - p0[1]) * frac;
        // Tangent direction
        let dx = p1[0] - p0[0];
        let dy = p1[1] - p0[1];
        let dl = (dx * dx + dy * dy).sqrt().max(1e-15);
        let nx = -dy / dl;
        let ny = dx / dl;
        pts.push([x + nx * hw, y + ny * hw, 0.0]);
        pts.push([x - nx * hw, y - ny * hw, 0.0]);
    }
    for i in 0..total_segs {
        let b = i * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn stepping_stones(positions: &[[f64; 2]], stone_radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for pos in positions {
        let c = pts.len();
        pts.push([pos[0], pos[1], 0.01]);
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([
                pos[0] + stone_radius * a.cos(),
                pos[1] + stone_radius * a.sin(),
                0.0,
            ]);
        }
        for i in 0..res {
            let j = if i + 1 < res { c + 2 + i } else { c + 1 };
            polys.push_cell(&[c as i64, (c + 1 + i) as i64, j as i64]);
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
    fn test_curved() {
        let p = curved_path(&[[0.0, 0.0], [5.0, 2.0], [10.0, 0.0]], 1.0, 12);
        assert!(p.polys.num_cells() > 5);
    }
    #[test]
    fn test_stones() {
        let s = stepping_stones(&[[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]], 0.3, 8);
        assert_eq!(s.polys.num_cells(), 24);
    }
}
