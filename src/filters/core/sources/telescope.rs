//! Telescope tube geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn telescope(
    main_radius: f64,
    main_length: f64,
    finder_radius: f64,
    finder_length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main tube
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { main_length };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([main_radius * a.cos(), main_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Lens cap (front)
    let lc = pts.len();
    pts.push([0.0, 0.0, main_length]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[lc as i64, (res + i) as i64, (res + j) as i64]);
    }
    // Finder scope (smaller tube on top)
    let fb = pts.len();
    let fy = main_radius + finder_radius * 1.2;
    for ring in 0..=1 {
        let z = main_length * 0.3 + if ring == 0 { 0.0 } else { finder_length };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([finder_radius * a.cos(), fy + finder_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (fb + i) as i64,
            (fb + j) as i64,
            (fb + res + j) as i64,
            (fb + res + i) as i64,
        ]);
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
        let t = telescope(0.15, 1.0, 0.03, 0.3, 12);
        assert!(t.polys.num_cells() > 20);
    }
}
