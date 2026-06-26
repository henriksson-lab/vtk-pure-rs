//! Lens (biconvex) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a biconvex lens shape by intersecting two spheres.
///
/// `radius` is the lens radius, `curvature` controls the sphere radius
/// (higher = flatter), `resolution` controls mesh density.
pub fn lens(radius: f64, curvature: f64, resolution: usize) -> PolyData {
    let n_theta = resolution.max(4);
    let n_phi = resolution.max(8);
    let radius = radius.abs();
    let sphere_r = (curvature.abs() * radius).max(radius);
    // Distance from center to sphere center
    let d = (sphere_r * sphere_r - radius * radius).sqrt();

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Front surface (partial sphere)
    let max_theta = (radius / sphere_r).asin();
    let front_offset = points.len();
    for j in 0..=n_theta {
        let theta = max_theta * j as f64 / n_theta as f64;
        for i in 0..=n_phi {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / n_phi as f64;
            let x = sphere_r * theta.sin() * phi.cos();
            let y = sphere_r * theta.sin() * phi.sin();
            let z = sphere_r * (1.0 - theta.cos()) - (sphere_r - d);
            points.push([x, y, z]);
        }
    }

    // Back surface (mirror)
    let back_offset = points.len();
    for j in 0..=n_theta {
        let theta = max_theta * j as f64 / n_theta as f64;
        for i in 0..=n_phi {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / n_phi as f64;
            let x = sphere_r * theta.sin() * phi.cos();
            let y = sphere_r * theta.sin() * phi.sin();
            let z = -(sphere_r * (1.0 - theta.cos()) - (sphere_r - d));
            points.push([x, y, z]);
        }
    }

    let row = n_phi + 1;

    // Triangulate front
    for j in 0..n_theta {
        for i in 0..n_phi {
            let p0 = (front_offset + j * row + i) as i64;
            let p1 = p0 + 1;
            let p2 = p0 + row as i64 + 1;
            let p3 = p0 + row as i64;
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p3]);
        }
    }

    // Triangulate back
    for j in 0..n_theta {
        for i in 0..n_phi {
            let p0 = (back_offset + j * row + i) as i64;
            let p1 = p0 + 1;
            let p2 = p0 + row as i64 + 1;
            let p3 = p0 + row as i64;
            polys.push_cell(&[p0, p2, p1]);
            polys.push_cell(&[p0, p3, p2]);
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
    fn basic_lens() {
        let l = lens(1.0, 2.0, 8);
        assert!(l.points.len() > 50);
        assert!(l.polys.num_cells() > 50);
    }

    #[test]
    fn flat_lens() {
        let l = lens(1.0, 10.0, 6); // very flat
        assert!(l.polys.num_cells() > 0);
    }
}
