//! Plane with circular or rectangular hole.

use crate::data::{CellArray, Points, PolyData};

/// Create a plane with a circular hole at center.
pub fn plane_with_circular_hole(
    width: f64,
    height: f64,
    hole_radius: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let hw = width * 0.5;
    let hh = height * 0.5;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let tx = if cos_angle.abs() > 0.0 {
            hw / cos_angle.abs()
        } else {
            f64::INFINITY
        };
        let ty = if sin_angle.abs() > 0.0 {
            hh / sin_angle.abs()
        } else {
            f64::INFINITY
        };
        let t = tx.min(ty);
        pts.push([t * cos_angle, t * sin_angle, 0.0]);
        pts.push([hole_radius * cos_angle, hole_radius * sin_angle, 0.0]);
    }

    for i in 0..res {
        let next = (i + 1) % res;
        polys.push_cell(&[
            (2 * i) as i64,
            (2 * next) as i64,
            (2 * next + 1) as i64,
            (2 * i + 1) as i64,
        ]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a plane with a rectangular hole.
pub fn plane_with_rect_hole(width: f64, height: f64, hole_w: f64, hole_h: f64) -> PolyData {
    let hw = width * 0.5;
    let hh = height * 0.5;
    let ihw = hole_w * 0.5;
    let ihh = hole_h * 0.5;

    let mut pts = Points::<f64>::new();
    // Outer corners
    pts.push([-hw, -hh, 0.0]); // 0
    pts.push([hw, -hh, 0.0]); // 1
    pts.push([hw, hh, 0.0]); // 2
    pts.push([-hw, hh, 0.0]); // 3
                              // Inner corners
    pts.push([-ihw, -ihh, 0.0]); // 4
    pts.push([ihw, -ihh, 0.0]); // 5
    pts.push([ihw, ihh, 0.0]); // 6
    pts.push([-ihw, ihh, 0.0]); // 7

    let mut polys = CellArray::new();
    // Bottom strip
    polys.push_cell(&[0, 1, 5, 4]);
    // Right strip
    polys.push_cell(&[1, 2, 6, 5]);
    // Top strip
    polys.push_cell(&[2, 3, 7, 6]);
    // Left strip
    polys.push_cell(&[3, 0, 4, 7]);

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_circular() {
        let p = plane_with_circular_hole(10.0, 10.0, 2.0, 16);
        assert_eq!(p.points.len(), 32);
        assert_eq!(p.polys.num_cells(), 16);
    }
    #[test]
    fn test_rect() {
        let p = plane_with_rect_hole(10.0, 10.0, 4.0, 4.0);
        assert_eq!(p.points.len(), 8);
        assert_eq!(p.polys.num_cells(), 4);
    }
}
