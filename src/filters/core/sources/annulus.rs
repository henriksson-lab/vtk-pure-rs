//! Annulus (flat ring) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a flat annulus (ring) in the XY plane.
pub fn annulus(inner_radius: f64, outer_radius: f64, resolution: usize, z: f64) -> PolyData {
    let n = resolution.max(4);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        points.push([inner_radius * cos_a, inner_radius * sin_a, z]);
        points.push([outer_radius * cos_a, outer_radius * sin_a, z]);
    }

    for i in 0..n {
        let next = (i + 1) % n;
        let i0 = (i * 2) as i64;
        let o0 = (i * 2 + 1) as i64;
        let i1 = (next * 2) as i64;
        let o1 = (next * 2 + 1) as i64;
        polys.push_cell(&[i0, o0, o1, i1]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let a = annulus(0.5, 1.0, 16, 0.0);
        assert_eq!(a.points.len(), 32);
        assert_eq!(a.polys.num_cells(), 16);
        assert_eq!(a.polys.cell(0), &[0, 1, 3, 2]);
        assert_eq!(a.polys.cell(15), &[30, 31, 1, 0]);

        // Check radii
        for i in 0..a.points.len() {
            let p = a.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!(r >= 0.49 && r <= 1.01, "r={r}");
        }
    }
}
