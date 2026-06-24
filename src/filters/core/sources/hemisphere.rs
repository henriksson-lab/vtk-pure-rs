//! Hemisphere geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Parameters for hemisphere generation.
pub struct HemisphereParams {
    /// Radius. Default: 1.0
    pub radius: f64,
    /// Number of latitude divisions. Default: 16
    pub theta_resolution: usize,
    /// Number of longitude divisions. Default: 32
    pub phi_resolution: usize,
    /// Whether to cap the base. Default: true
    pub cap: bool,
}

impl Default for HemisphereParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            theta_resolution: 16,
            phi_resolution: 32,
            cap: true,
        }
    }
}

/// Generate a hemisphere (upper half of a sphere).
pub fn hemisphere(params: &HemisphereParams) -> PolyData {
    let n_theta = params.theta_resolution;
    let n_phi = params.phi_resolution;

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Generate hemisphere points (theta from 0 to PI/2)
    for j in 0..=n_theta {
        let theta = std::f64::consts::FRAC_PI_2 * j as f64 / n_theta as f64;
        for i in 0..=n_phi {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / n_phi as f64;
            let x = params.radius * theta.sin() * phi.cos();
            let y = params.radius * theta.sin() * phi.sin();
            let z = params.radius * theta.cos();
            points.push([x, y, z]);
        }
    }

    // Triangulate
    let row = n_phi + 1;
    for j in 0..n_theta {
        for i in 0..n_phi {
            let p0 = (j * row + i) as i64;
            let p1 = p0 + 1;
            let p2 = p0 + row as i64 + 1;
            let p3 = p0 + row as i64;
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p3]);
        }
    }

    // Cap at the base (z=0 plane)
    if params.cap {
        let center_idx = points.len() as i64;
        points.push([0.0, 0.0, 0.0]);
        let base_row = n_theta * row;
        for i in 0..n_phi {
            polys.push_cell(&[center_idx, (base_row + i + 1) as i64, (base_row + i) as i64]);
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
    fn default_hemisphere() {
        let h = hemisphere(&HemisphereParams::default());
        assert!(h.points.len() > 50);
        assert!(h.polys.num_cells() > 50);
        // All z should be >= 0
        for i in 0..h.points.len() {
            assert!(h.points.get(i)[2] >= -1e-10);
        }
    }

    #[test]
    fn no_cap() {
        let h = hemisphere(&HemisphereParams {
            cap: false,
            ..Default::default()
        });
        let h_cap = hemisphere(&HemisphereParams {
            cap: true,
            ..Default::default()
        });
        assert!(h.polys.num_cells() < h_cap.polys.num_cells());
    }
}
