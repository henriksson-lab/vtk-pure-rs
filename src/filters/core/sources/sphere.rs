use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a UV sphere.
pub struct SphereParams {
    pub center: [f64; 3],
    pub radius: f64,
    pub theta_resolution: usize,
    pub phi_resolution: usize,
}

impl Default for SphereParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            radius: 0.5,
            theta_resolution: 8,
            phi_resolution: 8,
        }
    }
}

/// Generate a VTK-style sphere as PolyData with normals.
pub fn sphere(params: &SphereParams) -> PolyData {
    let n_theta = params.theta_resolution.max(3);
    let n_phi = params.phi_resolution.max(3);
    let [cx, cy, cz] = params.center;
    let r = params.radius;

    let phi_resolution = n_phi - 2;
    let n_pts = 2 + phi_resolution * n_theta;
    let n_tris = 2 * n_theta + 2 * n_theta * (phi_resolution - 1);

    let mut pts_flat = Vec::with_capacity(n_pts * 3);
    let mut nrm_flat = Vec::with_capacity(n_pts * 3);
    let mut conn = Vec::with_capacity(n_tris * 3);
    let mut offsets = Vec::with_capacity(n_tris + 1);
    offsets.push(0i64);

    // North pole
    pts_flat.extend_from_slice(&[cx, cy, cz + r]);
    nrm_flat.extend_from_slice(&[0.0, 0.0, 1.0]);

    // South pole
    pts_flat.extend_from_slice(&[cx, cy, cz - r]);
    nrm_flat.extend_from_slice(&[0.0, 0.0, -1.0]);

    let delta_phi = PI / (n_phi - 1) as f64;
    let delta_theta = 2.0 * PI / n_theta as f64;

    for i in 0..n_theta {
        let theta = i as f64 * delta_theta;
        for j in 1..n_phi - 1 {
            let phi = j as f64 * delta_phi;
            let radius = r * phi.sin();
            let n = [radius * theta.cos(), radius * theta.sin(), r * phi.cos()];
            pts_flat.extend_from_slice(&[n[0] + cx, n[1] + cy, n[2] + cz]);

            let mut norm = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if norm == 0.0 {
                norm = 1.0;
            }
            nrm_flat.extend_from_slice(&[n[0] / norm, n[1] / norm, n[2] / norm]);
        }
    }

    let num_poles = 2;
    let base = phi_resolution * n_theta;

    // North cap
    for i in 0..n_theta {
        let p0 = (phi_resolution * i + num_poles) as i64;
        let p1 = ((phi_resolution * (i + 1)) % base + num_poles) as i64;
        conn.extend_from_slice(&[p0, p1, 0]);
        offsets.push(conn.len() as i64);
    }

    // South cap
    let num_offset = phi_resolution - 1 + num_poles;
    for i in 0..n_theta {
        let p0 = (phi_resolution * i + num_offset) as i64;
        let p2 = ((phi_resolution * (i + 1)) % base + num_offset) as i64;
        conn.extend_from_slice(&[p0, 1, p2]);
        offsets.push(conn.len() as i64);
    }

    // Bands between poles
    for i in 0..n_theta {
        for j in 0..phi_resolution - 1 {
            let p0 = (phi_resolution * i + j + num_poles) as i64;
            let p1 = p0 + 1;
            let p2 = ((phi_resolution * (i + 1) + j) % base + num_poles + 1) as i64;
            conn.extend_from_slice(&[p0, p1, p2]);
            offsets.push(conn.len() as i64);
            conn.extend_from_slice(&[p0, p2, p2 - 1]);
            offsets.push(conn.len() as i64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(pts_flat);
    pd.polys = CellArray::from_raw(offsets, conn);
    pd.point_data_mut()
        .add_array(DataArray::from_vec("Normals", nrm_flat, 3).into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sphere() {
        let pd = sphere(&SphereParams::default());
        assert_eq!(pd.points.len(), 2 + 6 * 8);
        assert_eq!(pd.polys.num_cells(), 96);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn minimal_sphere() {
        let pd = sphere(&SphereParams {
            theta_resolution: 3,
            phi_resolution: 3,
            ..Default::default()
        });
        assert!(pd.points.len() > 0);
        assert!(pd.polys.num_cells() > 0);
    }
}
