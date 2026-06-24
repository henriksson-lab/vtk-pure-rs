//! Horseshoe shape.
use crate::data::{CellArray, Points, PolyData};

pub fn horseshoe(radius: f64, thickness: f64, na: usize) -> PolyData {
    let na = na.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let open_angle = std::f64::consts::PI * 0.3; // opening at bottom
    let start = open_angle / 2.0 + std::f64::consts::PI / 2.0;
    let end = 2.0 * std::f64::consts::PI - open_angle / 2.0 + std::f64::consts::PI / 2.0;
    let inner_r = radius - thickness / 2.0;
    let outer_r = radius + thickness / 2.0;
    for i in 0..=na {
        let t = i as f64 / na as f64;
        let a = start + (end - start) * t;
        pts.push([outer_r * a.cos(), outer_r * a.sin(), 0.0]);
        pts.push([inner_r * a.cos(), inner_r * a.sin(), 0.0]);
        pts.push([outer_r * a.cos(), outer_r * a.sin(), thickness * 0.5]);
        pts.push([inner_r * a.cos(), inner_r * a.sin(), thickness * 0.5]);
    }
    for i in 0..na {
        let b = i * 4;
        // Top surface
        polys.push_cell(&[
            (b + 2) as i64,
            (b + 3) as i64,
            (b + 7) as i64,
            (b + 6) as i64,
        ]);
        // Outer wall
        polys.push_cell(&[b as i64, (b + 4) as i64, (b + 6) as i64, (b + 2) as i64]);
        // Inner wall
        polys.push_cell(&[
            (b + 1) as i64,
            (b + 3) as i64,
            (b + 7) as i64,
            (b + 5) as i64,
        ]);
        // Bottom surface
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 5) as i64, (b + 4) as i64]);
    }
    // Nail holes (small circles)
    let mut lines = CellArray::new();
    for &hole_t in &[0.3, 0.5, 0.7] {
        let a = start + (end - start) * hole_t;
        let hr = thickness * 0.15;
        let hc = [(radius * a.cos()), (radius * a.sin())];
        let hb = pts.len();
        for j in 0..8 {
            let ha = 2.0 * std::f64::consts::PI * j as f64 / 8.0;
            pts.push([
                hc[0] + hr * ha.cos(),
                hc[1] + hr * ha.sin(),
                thickness * 0.51,
            ]);
        }
        for j in 0..8 {
            lines.push_cell(&[(hb + j) as i64, (hb + (j + 1) % 8) as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_horseshoe() {
        let m = horseshoe(2.0, 0.5, 16);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 30);
    }
}
