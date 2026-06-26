//! Capped cylinder with configurable resolution.

use crate::data::{CellArray, Points, PolyData};

/// Create a cylinder with caps along Z axis, centered at origin.
pub fn cylinder_capped(radius: f64, height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let half_h = height / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Side rings
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), -half_h]);
    }
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), half_h]);
    }

    // Side quads
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }

    // Cap rings are duplicated, matching vtkCylinderSource's separate cap points.
    let bottom_cap = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), -half_h]);
    }
    let top_cap = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), half_h]);
    }
    let bottom: Vec<i64> = (0..res).rev().map(|i| (bottom_cap + i) as i64).collect();
    let top: Vec<i64> = (0..res).map(|i| (top_cap + i) as i64).collect();
    polys.push_cell(&bottom);
    polys.push_cell(&top);

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
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }

    let bottom_cap = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([bottom_radius * a.cos(), bottom_radius * a.sin(), -half_h]);
    }
    let top_cap = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([top_radius * a.cos(), top_radius * a.sin(), half_h]);
    }
    let bottom: Vec<i64> = (0..res).rev().map(|i| (bottom_cap + i) as i64).collect();
    let top: Vec<i64> = (0..res).map(|i| (top_cap + i) as i64).collect();
    polys.push_cell(&bottom);
    polys.push_cell(&top);

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
        assert_eq!(c.points.len(), 48); // side rings + separate cap rings
        assert_eq!(c.polys.num_cells(), 14); // 12 side + 2 caps
    }
    #[test]
    fn test_tapered() {
        let c = tapered_cylinder(1.0, 0.5, 3.0, 8);
        assert_eq!(c.points.len(), 32);
        assert_eq!(c.polys.num_cells(), 10);
    }
}
