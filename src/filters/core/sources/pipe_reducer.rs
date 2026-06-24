//! Pipe reducer (concentric transition between two diameters).
use crate::data::{CellArray, Points, PolyData};
pub fn pipe_reducer(
    large_radius: f64,
    small_radius: f64,
    length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let nz = 10;
    for iz in 0..=nz {
        let t = iz as f64 / nz as f64;
        let r = large_radius * (1.0 - t) + small_radius * t;
        let z = t * length;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for iz in 0..nz {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (iz * res + i) as i64,
                (iz * res + j) as i64,
                ((iz + 1) * res + j) as i64,
                ((iz + 1) * res + i) as i64,
            ]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn eccentric_reducer(
    large_radius: f64,
    small_radius: f64,
    length: f64,
    offset: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let nz = 10;
    for iz in 0..=nz {
        let t = iz as f64 / nz as f64;
        let r = large_radius * (1.0 - t) + small_radius * t;
        let z = t * length;
        let cx = offset * t;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([cx + r * a.cos(), r * a.sin(), z]);
        }
    }
    for iz in 0..nz {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (iz * res + i) as i64,
                (iz * res + j) as i64,
                ((iz + 1) * res + j) as i64,
                ((iz + 1) * res + i) as i64,
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
    fn test_concentric() {
        let r = pipe_reducer(2.0, 1.0, 3.0, 12);
        assert!(r.points.len() > 50);
    }
    #[test]
    fn test_eccentric() {
        let r = eccentric_reducer(2.0, 1.0, 3.0, 0.5, 12);
        assert!(r.points.len() > 50);
    }
}
