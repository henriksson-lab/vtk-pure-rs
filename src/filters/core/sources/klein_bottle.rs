use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a Klein bottle (figure-8 immersion in 3D).
pub struct KleinBottleParams {
    pub center: [f64; 3],
    pub radius: f64,
    pub u_resolution: usize,
    pub v_resolution: usize,
}

impl Default for KleinBottleParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
            u_resolution: 32,
            v_resolution: 16,
        }
    }
}

/// Generate a Klein bottle as PolyData using the figure-8 immersion parametrization.
///
/// The figure-8 Klein bottle immersion is:
///   x = (a + cos(v/2) sin(u) - sin(v/2) sin(2u)) cos(v)
///   y = (a + cos(v/2) sin(u) - sin(v/2) sin(2u)) sin(v)
///   z = sin(v/2) sin(u) + cos(v/2) sin(2u)
/// where u in [0, 2pi), v in [0, 2pi), and a = radius.
pub fn klein_bottle(params: &KleinBottleParams) -> PolyData {
    let nu = params.u_resolution.max(3);
    let nv = params.v_resolution.max(3);
    let [cx, cy, cz] = params.center;
    let a = params.radius;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Generate vertices
    for j in 0..nv {
        let v = 2.0 * PI * j as f64 / nv as f64;
        let cv = v.cos();
        let sv = v.sin();
        let cv2 = (v / 2.0).cos();
        let sv2 = (v / 2.0).sin();

        for i in 0..nu {
            let u = 2.0 * PI * i as f64 / nu as f64;
            let su = u.sin();
            let s2u = (2.0 * u).sin();

            let r = a + cv2 * su - sv2 * s2u;
            let x = cx + r * cv;
            let y = cy + r * sv;
            let z = cz + sv2 * su + cv2 * s2u;

            points.push([x, y, z]);

            // Approximate normal via finite differences
            let eps = 1e-5;

            let u2 = u + eps;
            let su2 = u2.sin();
            let s2u2 = (2.0 * u2).sin();
            let r_u = a + cv2 * su2 - sv2 * s2u2;
            let du = [
                r_u * cv - r * cv,
                r_u * sv - r * sv,
                sv2 * su2 + cv2 * s2u2 - (sv2 * su + cv2 * s2u),
            ];

            let v2 = v + eps;
            let cv2_ = v2.cos();
            let sv2_ = v2.sin();
            let cv22 = (v2 / 2.0).cos();
            let sv22 = (v2 / 2.0).sin();
            let r_v = a + cv22 * su - sv22 * s2u;
            let dv = [
                r_v * cv2_ - r * cv,
                r_v * sv2_ - r * sv,
                sv22 * su + cv22 * s2u - (sv2 * su + cv2 * s2u),
            ];

            // Cross product du x dv
            let nx = du[1] * dv[2] - du[2] * dv[1];
            let ny = du[2] * dv[0] - du[0] * dv[2];
            let nz = du[0] * dv[1] - du[1] * dv[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-12);
            normals.push_tuple(&[nx / len, ny / len, nz / len]);
        }
    }

    // Generate quads (both u and v wrap around)
    for j in 0..nv {
        let j_next = (j + 1) % nv;
        for i in 0..nu {
            let i_next = (i + 1) % nu;
            let p00 = (j * nu + i) as i64;
            let p10 = (j * nu + i_next) as i64;
            let p01 = (j_next * nu + i) as i64;
            let p11 = (j_next * nu + i_next) as i64;
            polys.push_cell(&[p00, p10, p11, p01]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_klein_bottle() {
        let pd = klein_bottle(&KleinBottleParams::default());
        assert_eq!(pd.points.len(), 32 * 16);
        assert_eq!(pd.polys.num_cells(), 32 * 16);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn minimal_klein_bottle() {
        let pd = klein_bottle(&KleinBottleParams {
            u_resolution: 3,
            v_resolution: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 9);
        assert_eq!(pd.polys.num_cells(), 9);
    }
}
