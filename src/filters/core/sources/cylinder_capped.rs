//! Capped cylinder with configurable resolution.

use crate::data::{CellArray, Points, PolyData};

/// Create a cylinder with caps along Z axis, centered at origin.
pub fn cylinder_capped(radius: f64, height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let half_h = height / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Bottom ring
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), -half_h]);
    }
    // Top ring
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), half_h]);
    }
    // Centers
    let bot_center = pts.len();
    pts.push([0.0, 0.0, -half_h]);
    let top_center = pts.len();
    pts.push([0.0, 0.0, half_h]);

    // Side quads
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Bottom cap
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[bot_center as i64, j as i64, i as i64]);
    }
    // Top cap
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[top_center as i64, (res + i) as i64, (res + j) as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a tapered cylinder (different top and bottom radii).
pub fn tapered_cylinder(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let half_h = height / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([bottom_radius * a.cos(), bottom_radius * a.sin(), -half_h]);
    }
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([top_radius * a.cos(), top_radius * a.sin(), half_h]);
    }
    let bot_c = pts.len();
    pts.push([0.0, 0.0, -half_h]);
    let top_c = pts.len();
    pts.push([0.0, 0.0, half_h]);

    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[bot_c as i64, j as i64, i as i64]);
        polys.push_cell(&[top_c as i64, (res + i) as i64, (res + j) as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_capped() {
        let c = cylinder_capped(1.0, 2.0, 12);
        assert_eq!(c.points.len(), 26); // 24 + 2 centers
        assert_eq!(c.polys.num_cells(), 36); // 12 side + 12 bottom + 12 top
    }
    #[test]
    fn test_tapered() {
        let c = tapered_cylinder(1.0, 0.5, 3.0, 8);
        assert_eq!(c.points.len(), 18);
        assert_eq!(c.polys.num_cells(), 24);
    }
}
