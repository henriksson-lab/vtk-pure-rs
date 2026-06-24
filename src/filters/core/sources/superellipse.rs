//! Superellipse (Lamé curve) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a superellipse in the XY plane.
///
/// |x/a|^n + |y/b|^n = 1
/// n=2: ellipse, n<2: pinched, n>2: rounded rectangle
pub fn superellipse(a: f64, b: f64, exponent: f64, resolution: usize) -> PolyData {
    let n = resolution.max(8);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Center
    points.push([0.0, 0.0, 0.0]);

    for i in 0..=n {
        let t = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let cos_t = t.cos();
        let sin_t = t.sin();
        let x = a * cos_t.abs().powf(2.0 / exponent) * cos_t.signum();
        let y = b * sin_t.abs().powf(2.0 / exponent) * sin_t.signum();
        points.push([x, y, 0.0]);
    }

    // Fan triangulation
    for i in 0..n {
        polys.push_cell(&[0, (i + 1) as i64, (i + 2) as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Generate a 3D superellipsoid (generalized ellipsoid).
///
/// |x/a|^(2/e1) + |y/b|^(2/e1) + |z/c|^(2/e2) = 1
pub fn superellipsoid(a: f64, b: f64, c: f64, e1: f64, e2: f64, resolution: usize) -> PolyData {
    let n_u = resolution.max(4);
    let n_v = resolution.max(4);

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for j in 0..=n_v {
        let v = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * j as f64 / n_v as f64;
        for i in 0..=n_u {
            let u = -std::f64::consts::PI + 2.0 * std::f64::consts::PI * i as f64 / n_u as f64;

            let cos_v = v.cos();
            let sin_v = v.sin();
            let cos_u = u.cos();
            let sin_u = u.sin();

            let x =
                a * cos_v.abs().powf(e2) * cos_v.signum() * cos_u.abs().powf(e1) * cos_u.signum();
            let y =
                b * cos_v.abs().powf(e2) * cos_v.signum() * sin_u.abs().powf(e1) * sin_u.signum();
            let z = c * sin_v.abs().powf(e2) * sin_v.signum();

            points.push([x, y, z]);
        }
    }

    let row = n_u + 1;
    for j in 0..n_v {
        for i in 0..n_u {
            let p0 = (j * row + i) as i64;
            let p1 = p0 + 1;
            let p2 = p0 + row as i64 + 1;
            let p3 = p0 + row as i64;
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p3]);
        }
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
    fn circle() {
        let s = superellipse(1.0, 1.0, 2.0, 32);
        assert!(s.points.len() > 30);
        assert_eq!(s.polys.num_cells(), 32);
    }

    #[test]
    fn squircle() {
        let s = superellipse(1.0, 1.0, 4.0, 32);
        assert!(s.polys.num_cells() > 0);
    }

    #[test]
    fn superellipsoid_3d() {
        let s = superellipsoid(1.0, 1.0, 1.0, 1.0, 1.0, 8);
        assert!(s.points.len() > 50);
        assert!(s.polys.num_cells() > 50);
    }

    #[test]
    fn box_like() {
        let s = superellipsoid(1.0, 1.0, 1.0, 0.2, 0.2, 8);
        assert!(s.polys.num_cells() > 0);
    }
}
