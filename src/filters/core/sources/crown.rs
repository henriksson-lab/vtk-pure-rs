//! Crown/tiara geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn crown(
    radius: f64,
    height: f64,
    num_points: usize,
    point_height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(num_points * 2);
    let ncp = num_points.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base ring
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    // Top ring with peaks
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        let peak_position = i as f64 * ncp as f64 / res as f64;
        let distance_to_peak = (peak_position - peak_position.round()).abs();
        let peak = (1.0 - 2.0 * distance_to_peak).max(0.0);
        let z = height + point_height * peak;
        pts.push([radius * a.cos(), radius * a.sin(), z]);
    }
    // Side quads
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
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
        let c = crown(2.0, 1.0, 5, 0.8, 20);
        assert_eq!(c.polys.num_cells(), 20);
    }
}
