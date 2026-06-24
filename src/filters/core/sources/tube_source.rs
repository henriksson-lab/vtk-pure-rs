//! Parametric tube (hollow cylinder) source.

use crate::data::{CellArray, Points, PolyData};

/// Create a tube (hollow cylinder) with inner and outer radius.
pub fn tube_source(
    inner_radius: f64,
    outer_radius: f64,
    height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let half_h = height * 0.5;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Bottom ring: outer then inner
    // Top ring: outer then inner
    for i in 0..res {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        let co = angle.cos();
        let si = angle.sin();
        pts.push([outer_radius * co, outer_radius * si, -half_h]); // bottom outer
        pts.push([inner_radius * co, inner_radius * si, -half_h]); // bottom inner
        pts.push([outer_radius * co, outer_radius * si, half_h]); // top outer
        pts.push([inner_radius * co, inner_radius * si, half_h]); // top inner
    }

    for i in 0..res {
        let j = (i + 1) % res;
        let bo = i * 4; // bottom outer
        let bi = i * 4 + 1; // bottom inner
        let to = i * 4 + 2; // top outer
        let ti = i * 4 + 3; // top inner
        let nbo = j * 4;
        let nbi = j * 4 + 1;
        let nto = j * 4 + 2;
        let nti = j * 4 + 3;

        // Outer surface
        polys.push_cell(&[bo as i64, nbo as i64, nto as i64, to as i64]);
        // Inner surface (reversed winding)
        polys.push_cell(&[bi as i64, ti as i64, nti as i64, nbi as i64]);
        // Bottom annulus
        polys.push_cell(&[bo as i64, bi as i64, nbi as i64, nbo as i64]);
        // Top annulus
        polys.push_cell(&[to as i64, nto as i64, nti as i64, ti as i64]);
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
    fn test_tube() {
        let t = tube_source(0.5, 1.0, 2.0, 12);
        assert_eq!(t.points.len(), 48); // 12 * 4
        assert_eq!(t.polys.num_cells(), 48); // 12 * 4 faces
    }
    #[test]
    fn test_tube_min_res() {
        let t = tube_source(0.3, 0.5, 1.0, 3);
        assert_eq!(t.points.len(), 12);
        assert_eq!(t.polys.num_cells(), 12);
    }
}
