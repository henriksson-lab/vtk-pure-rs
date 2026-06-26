//! Gear rack (linear gear teeth) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn gear_rack(
    num_teeth: usize,
    tooth_height: f64,
    tooth_width: f64,
    base_height: f64,
    thickness: f64,
    length: f64,
) -> PolyData {
    let nt = num_teeth.max(1);
    let tw = length / nt as f64;
    let tooth_width = tooth_width.clamp(0.0, tw);
    let tooth_start = (tw - tooth_width) * 0.5;
    let tooth_end = tooth_start + tooth_width;
    let ht = thickness / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Profile: zigzag teeth on top of a rectangle
    let mut profile = Vec::new();
    profile.push([0.0, 0.0]);
    for i in 0..nt {
        let x = i as f64 * tw;
        profile.push([x, base_height]);
        profile.push([x + tooth_start, base_height + tooth_height]);
        profile.push([x + tooth_end, base_height + tooth_height]);
        profile.push([x + tw, base_height]);
    }
    profile.push([length, 0.0]);
    let np = profile.len();
    for p in &profile {
        pts.push([p[0], p[1], -ht]);
    }
    for p in &profile {
        pts.push([p[0], p[1], ht]);
    }
    // Front face
    for i in 1..np - 1 {
        polys.push_cell(&[0, (i + 1) as i64, i as i64]);
    }
    // Back face
    for i in 1..np - 1 {
        polys.push_cell(&[np as i64, (np + i) as i64, (np + i + 1) as i64]);
    }
    // Sides
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[i as i64, j as i64, (np + j) as i64, (np + i) as i64]);
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
        let g = gear_rack(5, 0.3, 0.5, 0.5, 0.2, 5.0);
        assert!(g.points.len() > 20);
        assert!(g.polys.num_cells() > 10);
    }

    #[test]
    fn tooth_width_controls_flat() {
        let g = gear_rack(1, 0.3, 0.2, 0.5, 0.2, 1.0);
        assert!((g.points.get(2)[0] - 0.4).abs() < 1e-10);
        assert!((g.points.get(3)[0] - 0.6).abs() < 1e-10);
    }
}
