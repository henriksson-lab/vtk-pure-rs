//! Chess piece geometry (pawn, rook profiles via revolution).
use crate::data::{CellArray, Points, PolyData};
pub fn pawn(height: f64, resolution: usize) -> PolyData {
    let h = height;
    let res = resolution.max(8);
    let profile: Vec<[f64; 2]> = vec![
        [h * 0.25, 0.0],
        [h * 0.2, h * 0.02],
        [h * 0.12, h * 0.05],
        [h * 0.08, h * 0.3],
        [h * 0.06, h * 0.5],
        [h * 0.08, h * 0.55],
        [h * 0.1, h * 0.6],
        [h * 0.12, h * 0.65],
        [h * 0.08, h * 0.75],
        [h * 0.1, h * 0.85],
        [h * 0.08, h * 0.95],
        [h * 0.0, h * 1.0],
    ];
    revolve(&profile, res)
}
pub fn rook(height: f64, resolution: usize) -> PolyData {
    let h = height;
    let res = resolution.max(8);
    let profile: Vec<[f64; 2]> = vec![
        [h * 0.2, 0.0],
        [h * 0.18, h * 0.03],
        [h * 0.1, h * 0.05],
        [h * 0.1, h * 0.7],
        [h * 0.12, h * 0.72],
        [h * 0.14, h * 0.75],
        [h * 0.14, h * 0.9],
        [h * 0.12, h * 0.9],
        [h * 0.12, h * 0.95],
        [h * 0.15, h * 0.95],
        [h * 0.15, h * 1.0],
        [h * 0.0, h * 1.0],
    ];
    revolve(&profile, res)
}
fn revolve(profile: &[[f64; 2]], res: usize) -> PolyData {
    let np = profile.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iu in 0..res {
        let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
        for p in profile {
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
    fn test_pawn() {
        let p = pawn(3.0, 12);
        assert!(p.points.len() > 50);
        assert!(p.polys.num_cells() > 50);
    }
    #[test]
    fn test_rook() {
        let r = rook(3.0, 12);
        assert!(r.points.len() > 50);
    }
}
