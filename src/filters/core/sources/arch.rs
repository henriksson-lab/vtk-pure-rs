//! Arch and vault geometry sources.
use crate::data::{CellArray, Points, PolyData};
/// Create a semicircular arch.
pub fn arch(inner_radius: f64, outer_radius: f64, thickness: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let half_t = thickness / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..=res {
        let a = std::f64::consts::PI * i as f64 / res as f64;
        let ci = a.cos();
        let si = a.sin();
        pts.push([inner_radius * ci, inner_radius * si, -half_t]);
        pts.push([outer_radius * ci, outer_radius * si, -half_t]);
        pts.push([inner_radius * ci, inner_radius * si, half_t]);
        pts.push([outer_radius * ci, outer_radius * si, half_t]);
    }
    for i in 0..res {
        let b = i * 4;
        // Front face (inner-outer quad)
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 5) as i64, (b + 4) as i64]);
        // Back face
        polys.push_cell(&[
            (b + 2) as i64,
            (b + 6) as i64,
            (b + 7) as i64,
            (b + 3) as i64,
        ]);
        // Outer surface
        polys.push_cell(&[
            (b + 1) as i64,
            (b + 3) as i64,
            (b + 7) as i64,
            (b + 5) as i64,
        ]);
        // Inner surface
        polys.push_cell(&[b as i64, (b + 4) as i64, (b + 6) as i64, (b + 2) as i64]);
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
    fn test_arch() {
        let a = arch(1.0, 1.5, 0.5, 12);
        assert!(a.points.len() > 20);
        assert!(a.polys.num_cells() > 20);
    }
}
