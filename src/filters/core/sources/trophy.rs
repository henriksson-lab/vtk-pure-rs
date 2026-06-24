//! Trophy/cup shaped geometry (surface of revolution).
use crate::data::{CellArray, Points, PolyData};
pub fn trophy(height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let h = height;
    let profile: Vec<[f64; 2]> = vec![
        [h * 0.3, 0.0],
        [h * 0.25, h * 0.02],
        [h * 0.1, h * 0.05],
        [h * 0.08, h * 0.15],
        [h * 0.08, h * 0.45],
        [h * 0.15, h * 0.5],
        [h * 0.25, h * 0.55],
        [h * 0.3, h * 0.7],
        [h * 0.28, h * 0.85],
        [h * 0.22, h * 0.95],
        [h * 0.25, h * 1.0],
    ];
    let np = profile.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iu in 0..res {
        let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
        for p in &profile {
            pts.push([p[0] * a.cos(), p[0] * a.sin(), p[1]]);
        }
    }
    for iu in 0..res {
        let iu1 = (iu + 1) % res;
        for ip in 0..np - 1 {
            polys.push_cell(&[
                (iu * np + ip) as i64,
                (iu * np + ip + 1) as i64,
                (iu1 * np + ip + 1) as i64,
                (iu1 * np + ip) as i64,
            ]);
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
        let t = trophy(3.0, 12);
        assert!(t.points.len() > 50);
        assert!(t.polys.num_cells() > 50);
    }
}
