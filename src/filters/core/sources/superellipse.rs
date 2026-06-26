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
/// `e1` controls squareness in the z axis; `e2` controls squareness in the x-y plane.
pub fn superellipsoid(a: f64, b: f64, c: f64, e1: f64, e2: f64, resolution: usize) -> PolyData {
    let n_u = resolution.max(4);
    let n_v = resolution.max(4);

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for j in 0..=n_v {
        let v = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * j as f64 / n_v as f64;
        for i in 0..=n_u {
            let u = -std::f64::consts::PI + 2.0 * std::f64::consts::PI * i as f64 / n_u as f64;

            let tmp = signed_pow(v.cos(), e1);
            let x = a * tmp * signed_pow(u.sin(), e2);
            let y = b * tmp * signed_pow(u.cos(), e2);
            let z = c * signed_pow(v.sin(), e1);

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

fn signed_pow(value: f64, exponent: f64) -> f64 {
    const EPS: f64 = 1.0e-6;
    if value == 0.0 {
        return 0.0;
    }
    if exponent == 0.0 {
        return 1.0;
    }
    if value.abs() <= EPS {
        return 0.0;
    }
    value.signum() * value.abs().powf(exponent)
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
