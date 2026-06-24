//! Torus knot geometry source: parametric curves on a torus surface.

use crate::data::{CellArray, Points, PolyData};

/// Generate a (p,q) torus knot as a tube geometry.
///
/// p = number of times the knot wraps around the torus longitudinally
/// q = number of times it wraps meridionally
pub fn torus_knot(
    p: usize,
    q: usize,
    major_radius: f64,
    minor_radius: f64,
    tube_radius: f64,
    resolution: usize,
    tube_sides: usize,
) -> PolyData {
    let n = resolution.max(16);
    let ts = tube_sides.max(3);

    // Generate knot centerline
    let mut centers = Vec::with_capacity(n);
    let mut tangents = Vec::with_capacity(n);

    for i in 0..n {
        let t = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let r = major_radius + minor_radius * (q as f64 * t).cos();
        let x = r * (p as f64 * t).cos();
        let y = r * (p as f64 * t).sin();
        let z = minor_radius * (q as f64 * t).sin();
        centers.push([x, y, z]);

        // Tangent via finite difference
        let dt = 0.001;
        let t2 = t + dt;
        let r2 = major_radius + minor_radius * (q as f64 * t2).cos();
        let dx = r2 * (p as f64 * t2).cos() - x;
        let dy = r2 * (p as f64 * t2).sin() - y;
        let dz = minor_radius * (q as f64 * t2).sin() - z;
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-15);
        tangents.push([dx / len, dy / len, dz / len]);
    }

    // Generate tube around centerline
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..n {
        let t = &tangents[i];
        let up = if t[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = cross(*t, up);
        let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt().max(1e-15);
        let u = [u[0] / ul, u[1] / ul, u[2] / ul];
        let v = cross(*t, u);

        for j in 0..ts {
            let angle = 2.0 * std::f64::consts::PI * j as f64 / ts as f64;
            let px = centers[i][0] + tube_radius * (u[0] * angle.cos() + v[0] * angle.sin());
            let py = centers[i][1] + tube_radius * (u[1] * angle.cos() + v[1] * angle.sin());
            let pz = centers[i][2] + tube_radius * (u[2] * angle.cos() + v[2] * angle.sin());
            points.push([px, py, pz]);
        }
    }

    // Connect tube rings
    for i in 0..n {
        let i_next = (i + 1) % n;
        for j in 0..ts {
            let j_next = (j + 1) % ts;
            let p0 = (i * ts + j) as i64;
            let p1 = (i * ts + j_next) as i64;
            let p2 = (i_next * ts + j_next) as i64;
            let p3 = (i_next * ts + j) as i64;
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p3]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trefoil() {
        let k = torus_knot(2, 3, 2.0, 0.8, 0.1, 64, 6);
        assert!(k.points.len() > 100);
        assert!(k.polys.num_cells() > 100);
    }
    #[test]
    fn simple_knot() {
        let k = torus_knot(3, 2, 1.0, 0.5, 0.05, 32, 4);
        assert!(k.polys.num_cells() > 0);
    }
}
