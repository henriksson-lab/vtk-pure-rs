//! Catenary curve and surface geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a catenary curve: y = a * cosh(x/a).
pub fn catenary_curve(a: f64, x_range: (f64, f64), resolution: usize) -> PolyData {
    let n = resolution.max(4);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let ids: Vec<i64> = (0..=n)
        .map(|i| {
            let x = x_range.0 + (x_range.1 - x_range.0) * i as f64 / n as f64;
            let y = a * (x / a).cosh();
            pts.push([x, y, 0.0]);
            i as i64
        })
        .collect();
    lines.push_cell(&ids);
    let mut mesh = PolyData::new();
    mesh.points = pts;
    mesh.lines = lines;
    mesh
}

/// Generate a catenoid surface (minimal surface of revolution of a catenary).
pub fn catenoid(a: f64, height: f64, resolution: usize) -> PolyData {
    let n_u = resolution.max(4);
    let n_v = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for j in 0..=n_u {
        let v = -height / 2.0 + height * j as f64 / n_u as f64;
        for i in 0..=n_v {
            let u = 2.0 * std::f64::consts::PI * i as f64 / n_v as f64;
            let r = a * (v / a).cosh();
            pts.push([r * u.cos(), r * u.sin(), v]);
        }
    }

    let row = n_v + 1;
    for j in 0..n_u {
        for i in 0..n_v {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = pts;
    mesh.polys = polys;
    mesh
}

/// Generate a suspension bridge cable (catenary between two towers).
pub fn bridge_cable(span: f64, sag: f64, tower_height: f64, resolution: usize) -> PolyData {
    // Find catenary parameter a such that y(span/2) - y(0) = sag
    // a*cosh(span/(2*a)) - a = sag
    let mut a_param = span; // initial guess
    for _ in 0..50 {
        // Newton iteration
        let f = a_param * (span / (2.0 * a_param)).cosh() - a_param - sag;
        let fp = (span / (2.0 * a_param)).cosh()
            - span / (2.0 * a_param) * (span / (2.0 * a_param)).sinh()
            - 1.0;
        if fp.abs() > 1e-15 {
            a_param -= f / fp;
        }
    }

    let n = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let y_min = a_param;

    let ids: Vec<i64> = (0..=n)
        .map(|i| {
            let x = -span / 2.0 + span * i as f64 / n as f64;
            let y = tower_height - (a_param * (x / a_param).cosh() - y_min);
            pts.push([x, 0.0, y]);
            i as i64
        })
        .collect();
    lines.push_cell(&ids);
    let mut mesh = PolyData::new();
    mesh.points = pts;
    mesh.lines = lines;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catenary() {
        let c = catenary_curve(1.0, (-2.0, 2.0), 20);
        assert_eq!(c.points.len(), 21);
        assert_eq!(c.lines.num_cells(), 1);
    }
    #[test]
    fn catenoid_surf() {
        let c = catenoid(0.5, 2.0, 16);
        assert!(c.points.len() > 100);
        assert!(c.polys.num_cells() > 100);
    }
    #[test]
    fn bridge() {
        let c = bridge_cable(100.0, 10.0, 50.0, 32);
        assert_eq!(c.lines.num_cells(), 1);
        assert_eq!(c.points.len(), 33);
    }
}
