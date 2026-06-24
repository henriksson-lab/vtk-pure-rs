//! Simple arrow source (shaft + cone tip) along Z axis.

use crate::data::{CellArray, Points, PolyData};

/// Create an arrow along Z axis with given parameters.
pub fn arrow_z(
    shaft_radius: f64,
    shaft_length: f64,
    tip_radius: f64,
    tip_length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Shaft bottom ring
    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([shaft_radius * angle.cos(), shaft_radius * angle.sin(), 0.0]);
    }
    // Shaft top ring
    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([
            shaft_radius * angle.cos(),
            shaft_radius * angle.sin(),
            shaft_length,
        ]);
    }
    // Tip base ring
    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([
            tip_radius * angle.cos(),
            tip_radius * angle.sin(),
            shaft_length,
        ]);
    }
    // Tip apex
    let apex = pts.len();
    pts.push([0.0, 0.0, shaft_length + tip_length]);
    // Bottom center
    let bot = pts.len();
    pts.push([0.0, 0.0, 0.0]);

    // Shaft quads
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Tip triangles
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[(2 * res + i) as i64, (2 * res + j) as i64, apex as i64]);
    }
    // Bottom cap
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[bot as i64, j as i64, i as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a double-headed arrow along Z.
pub fn double_arrow_z(
    shaft_radius: f64,
    shaft_length: f64,
    tip_radius: f64,
    tip_length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let half = shaft_length * 0.5;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Shaft rings at -half and +half
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([shaft_radius * a.cos(), shaft_radius * a.sin(), -half]);
    }
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([shaft_radius * a.cos(), shaft_radius * a.sin(), half]);
    }
    // Tip bases
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([tip_radius * a.cos(), tip_radius * a.sin(), half]);
    }
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([tip_radius * a.cos(), tip_radius * a.sin(), -half]);
    }
    let top_apex = pts.len();
    pts.push([0.0, 0.0, half + tip_length]);
    let bot_apex = pts.len();
    pts.push([0.0, 0.0, -half - tip_length]);

    // Shaft
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Top tip
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[(2 * res + i) as i64, (2 * res + j) as i64, top_apex as i64]);
    }
    // Bottom tip
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[(3 * res + j) as i64, (3 * res + i) as i64, bot_apex as i64]);
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
    fn test_arrow() {
        let a = arrow_z(0.1, 1.0, 0.2, 0.3, 8);
        assert_eq!(a.points.len(), 26); // 3*8 + 2
        assert_eq!(a.polys.num_cells(), 24); // 8 shaft + 8 tip + 8 bottom
    }
    #[test]
    fn test_double() {
        let a = double_arrow_z(0.1, 1.0, 0.2, 0.3, 6);
        assert_eq!(a.points.len(), 26); // 4*6 + 2
    }
}
