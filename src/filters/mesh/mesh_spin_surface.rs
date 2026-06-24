//! Spin a polyline around an axis to create a surface of revolution.
use crate::data::{CellArray, Points, PolyData};
pub fn spin_polyline(profile_points: &[[f64; 2]], axis: usize, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let np = profile_points.len();
    if np < 2 {
        return PolyData::new();
    }
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iu in 0..res {
        let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
        let c = a.cos();
        let s = a.sin();
        for p in profile_points {
            match axis {
                0 => pts.push([p[1], p[0] * c, p[0] * s]),
                1 => pts.push([p[0] * c, p[1], p[0] * s]),
                _ => pts.push([p[0] * c, p[0] * s, p[1]]),
            }
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
    fn test_cylinder() {
        let profile = vec![[1.0, 0.0], [1.0, 2.0]];
        let r = spin_polyline(&profile, 2, 12);
        assert_eq!(r.points.len(), 24);
        assert_eq!(r.polys.num_cells(), 12);
    }
    #[test]
    fn test_vase() {
        let profile = vec![[1.0, 0.0], [0.5, 1.0], [0.8, 2.0], [1.2, 3.0]];
        let r = spin_polyline(&profile, 2, 16);
        assert_eq!(r.polys.num_cells(), 48);
    }
}
