//! Open cylinder (tube without caps) and cone frustum.

use crate::data::{CellArray, Points, PolyData};

/// Create an open cylinder (no caps) along Z axis.
pub fn open_cylinder(radius: f64, height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let half_h = height * 0.5;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        pts.push([x, y, -half_h]);
        pts.push([x, y, half_h]);
    }

    for i in 0..res {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        let j0 = ((i + 1) % res) * 2;
        let j1 = ((i + 1) % res) * 2 + 1;
        polys.push_cell(&[i0 as i64, j0 as i64, j1 as i64, i1 as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a cone frustum (truncated cone) along Z axis.
pub fn cone_frustum(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let half_h = height * 0.5;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([
            bottom_radius * angle.cos(),
            bottom_radius * angle.sin(),
            -half_h,
        ]);
        pts.push([top_radius * angle.cos(), top_radius * angle.sin(), half_h]);
    }

    // Side faces
    for i in 0..res {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        let j0 = ((i + 1) % res) * 2;
        let j1 = ((i + 1) % res) * 2 + 1;
        polys.push_cell(&[i0 as i64, j0 as i64, j1 as i64, i1 as i64]);
    }

    // Bottom cap
    let bottom_center = pts.len();
    pts.push([0.0, 0.0, -half_h]);
    for i in 0..res {
        let a = (i * 2) as i64;
        let b = (((i + 1) % res) * 2) as i64;
        polys.push_cell(&[bottom_center as i64, b, a]);
    }

    // Top cap
    let top_center = pts.len();
    pts.push([0.0, 0.0, half_h]);
    for i in 0..res {
        let a = (i * 2 + 1) as i64;
        let b = (((i + 1) % res) * 2 + 1) as i64;
        polys.push_cell(&[top_center as i64, a, b]);
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
    fn test_open_cyl() {
        let c = open_cylinder(1.0, 2.0, 16);
        assert_eq!(c.points.len(), 32);
        assert_eq!(c.polys.num_cells(), 16);
    }
    #[test]
    fn test_frustum() {
        let f = cone_frustum(2.0, 1.0, 3.0, 8);
        assert_eq!(f.points.len(), 18); // 16 side + 2 centers
        assert_eq!(f.polys.num_cells(), 24); // 8 side + 8 bottom + 8 top
    }
}
