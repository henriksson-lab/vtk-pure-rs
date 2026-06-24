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
            theta_resolution: 16,
            phi_resolution: 16,
        }
    }
}

/// Generate a UV sphere as PolyData with normals.
/// Uses pre-allocated flat buffers to avoid per-element Arc::make_mut overhead.
pub fn sphere(params: &SphereParams) -> PolyData {
    let n_theta = params.theta_resolution.max(3);
    let n_phi = params.phi_resolution.max(3);
    let [cx, cy, cz] = params.center;
    let r = params.radius;

    let n_pts = 2 + (n_phi - 1) * n_theta; // poles + rings
    let n_tris = n_theta + 2 * n_theta * (n_phi - 2) + n_theta; // caps + body

    // Pre-allocate flat buffers
    let mut pts_flat = Vec::with_capacity(n_pts * 3);
    let mut nrm_flat = Vec::with_capacity(n_pts * 3);
    let mut conn = Vec::with_capacity(n_tris * 3);
    let mut offsets = Vec::with_capacity(n_tris + 1);
    offsets.push(0i64);

    // North pole
    pts_flat.extend_from_slice(&[cx, cy, cz + r]);
    nrm_flat.extend_from_slice(&[0.0, 0.0, 1.0]);

    // Interior rings
    for j in 1..n_phi {
        let phi = PI * j as f64 / n_phi as f64;
        let sp = phi.sin();
        let cp = phi.cos();
        for i in 0..n_theta {
            let theta = 2.0 * PI * i as f64 / n_theta as f64;
            let (st, ct) = (theta.sin(), theta.cos());
            let (nx, ny, nz) = (sp * ct, sp * st, cp);
            pts_flat.extend_from_slice(&[cx + r * nx, cy + r * ny, cz + r * nz]);
            nrm_flat.extend_from_slice(&[nx, ny, nz]);
        }
    }

    // South pole
    pts_flat.extend_from_slice(&[cx, cy, cz - r]);
    nrm_flat.extend_from_slice(&[0.0, 0.0, -1.0]);
    let south_pole = (n_pts - 1) as i64;

    // North cap
    for i in 0..n_theta {
        let next = (i + 1) % n_theta;
        conn.extend_from_slice(&[0, (1 + i) as i64, (1 + next) as i64]);
        offsets.push(conn.len() as i64);
    }

    // Body quads → 2 triangles each
    for j in 0..n_phi - 2 {
        let rs = 1 + j * n_theta;
        let nrs = 1 + (j + 1) * n_theta;
        for i in 0..n_theta {
            let next = (i + 1) % n_theta;
            let (a, b, c, d) = (
                (rs + i) as i64,
                (nrs + i) as i64,
                (nrs + next) as i64,
                (rs + next) as i64,
            );
            conn.extend_from_slice(&[a, b, c]);
            offsets.push(conn.len() as i64);
            conn.extend_from_slice(&[a, c, d]);
            offsets.push(conn.len() as i64);
        }
    }

    // South cap
    let lrs = 1 + (n_phi - 2) * n_theta;
    for i in 0..n_theta {
        let next = (i + 1) % n_theta;
        conn.extend_from_slice(&[(lrs + i) as i64, south_pole, (lrs + next) as i64]);
        offsets.push(conn.len() as i64);
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
        // poles (2) + rings (14 * 16)
        assert_eq!(pd.points.len(), 2 + 15 * 16);
        assert!(pd.polys.num_cells() > 0);
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
