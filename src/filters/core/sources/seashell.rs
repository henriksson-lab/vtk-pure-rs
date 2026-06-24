//! Seashell (gastropod shell) parametric surface.

use crate::data::{CellArray, Points, PolyData};

/// Generate a seashell shape using a parametric model.
///
/// Based on the logarithmic spiral shell model with expanding aperture.
pub fn seashell(
    turns: f64,
    aperture_growth: f64,
    shell_radius: f64,
    tube_radius_start: f64,
    tube_radius_growth: f64,
    resolution_u: usize,
    resolution_v: usize,
) -> PolyData {
    let nu = resolution_u.max(8);
    let nv = resolution_v.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for j in 0..=nv {
        let v = 2.0 * std::f64::consts::PI * j as f64 / nv as f64; // around tube
        for i in 0..=nu {
            let u = turns * 2.0 * std::f64::consts::PI * i as f64 / nu as f64; // along spiral
            let r = shell_radius * (aperture_growth * u).exp();
            let tube_r = tube_radius_start * (tube_radius_growth * u).exp();

            // Spiral center
            let cx = r * u.cos();
            let cy = r * u.sin();
            let cz = r * 0.3; // vertical rise

            // Tube cross-section (perpendicular to spiral tangent)
            let tangent_angle = u + std::f64::consts::FRAC_PI_2;
            let nx = tangent_angle.cos();
            let ny = tangent_angle.sin();

            let px = cx + tube_r * (v.cos() * nx);
            let py = cy + tube_r * (v.cos() * ny);
            let pz = cz + tube_r * v.sin();

            pts.push([px, py, pz]);
        }
    }

    let row = nu + 1;
    for j in 0..nv {
        for i in 0..nu {
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

/// Generate a simplified nautilus shell.
pub fn nautilus(turns: f64, resolution: usize) -> PolyData {
    seashell(turns, 0.12, 0.2, 0.08, 0.12, resolution, resolution / 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_shell() {
        let s = seashell(3.0, 0.1, 0.5, 0.1, 0.1, 32, 12);
        assert!(s.points.len() > 200);
        assert!(s.polys.num_cells() > 200);
    }
    #[test]
    fn nautilus_shell() {
        let n = nautilus(2.5, 24);
        assert!(n.polys.num_cells() > 100);
    }
}
