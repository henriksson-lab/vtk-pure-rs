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

    // Corner points
    let c0 = pts.len();
    pts.push([-hw, -hh, 0.0]);
    let c1 = pts.len();
    pts.push([hw, -hh, 0.0]);
    let c2 = pts.len();
    pts.push([hw, hh, 0.0]);
    let c3 = pts.len();
    pts.push([-hw, hh, 0.0]);

    // Hole circle points
    let hole_start = pts.len();
    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([hole_radius * angle.cos(), hole_radius * angle.sin(), 0.0]);
    }

    // Connect corners to hole with triangles
    // Split into 4 quadrants
    let quarter = res / 4;
    let sections = [
        (c0, c1, 0, quarter),
        (c1, c2, quarter, 2 * quarter),
        (c2, c3, 2 * quarter, 3 * quarter),
        (c3, c0, 3 * quarter, res),
    ];

    for (ca, cb, start, end) in sections {
        // First triangle: corner_a -> first hole vertex
        polys.push_cell(&[ca as i64, (hole_start + start) as i64, cb as i64]);
        // Triangles along the hole arc
        for i in start..end {
            let next = if i + 1 >= res { 0 } else { i + 1 };
            polys.push_cell(&[
                cb as i64,
                (hole_start + i) as i64,
                (hole_start + next) as i64,
            ]);
        }
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
        assert!(p.points.len() >= 20);
        assert!(p.polys.num_cells() > 0);
    }
    #[test]
    fn test_rect() {
        let p = plane_with_rect_hole(10.0, 10.0, 4.0, 4.0);
        assert_eq!(p.points.len(), 8);
        assert_eq!(p.polys.num_cells(), 4);
    }
}
