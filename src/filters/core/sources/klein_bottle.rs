use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a Klein bottle (classical immersion in 3D).
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

/// Generate a Klein bottle using VTK's `vtkParametricKlein` parametrization.
///
/// VTK uses u in [0, PI], v in [0, 2*PI], no U join, and a V join.
pub fn klein_bottle(params: &KleinBottleParams) -> PolyData {
    let nu = params.u_resolution.max(3);
    let nv = params.v_resolution.max(3);
    let [cx, cy, cz] = params.center;
    let radius = params.radius;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    for j in 0..nv {
        let v = 2.0 * PI * j as f64 / nv as f64;
        for i in 0..=nu {
            let u = PI * i as f64 / nu as f64;
            let (pt, du, dv) = evaluate_klein(u, v);

            points.push([
                cx + radius * pt[0],
                cy + radius * pt[1],
                cz + radius * pt[2],
            ]);

            let nx = du[1] * dv[2] - du[2] * dv[1];
            let ny = du[2] * dv[0] - du[0] * dv[2];
            let nz = du[0] * dv[1] - du[1] * dv[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-12);
            normals.push_tuple(&[nx / len, ny / len, nz / len]);
        }
    }

    let cols = nu + 1;
    for j in 0..nv {
        let j_next = (j + 1) % nv;
        for i in 0..nu {
            let p00 = (j * cols + i) as i64;
            let p10 = (j * cols + i + 1) as i64;
            let p01 = (j_next * cols + i) as i64;
            let p11 = (j_next * cols + i + 1) as i64;
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

/// VTK `vtkParametricKlein::Evaluate`, translated with Rust names.
pub(crate) fn evaluate_klein(u: f64, v: f64) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let cu = u.cos();
    let su = u.sin();
    let cv = v.cos();
    let sv = v.sin();

    let cu2 = cu.powi(2);
    let cu3 = cu.powi(3);
    let cu4 = cu.powi(4);
    let cu5 = cu.powi(5);
    let cu6 = cu.powi(6);
    let cu7 = cu.powi(7);
    let cu8 = cu.powi(8);
    let su2 = su.powi(2);

    let sub_x = 3.0 * cv + 5.0 * su * cv * cu - 30.0 * su - 60.0 * su * cu6 + 90.0 * su * cu4;
    let sub_y = 80.0 * cv * cu7 * su + 48.0 * cv * cu6
        - 80.0 * cv * cu5 * su
        - 48.0 * cv * cu4
        - 5.0 * cv * cu3 * su
        - 3.0 * cv * cu2
        + 5.0 * su * cv * cu
        + 3.0 * cv
        - 60.0 * su;
    let sub_z = 3.0 + 5.0 * su * cu;

    let pt = [
        -2.0 / 15.0 * cu * sub_x,
        -1.0 / 15.0 * su * sub_y,
        2.0 / 15.0 * sv * sub_z,
    ];

    let du = [
        2.0 / 15.0 * su * sub_x
            - 2.0 / 15.0
                * cu
                * (5.0 * cv * cu2 - 5.0 * su2 * cv - 30.0 * cu - 60.0 * cu7
                    + 360.0 * su2 * cu5
                    + 90.0 * cu5
                    - 360.0 * su2 * cu3),
        -1.0 / 15.0 * cu * sub_y
            - 1.0 / 15.0
                * su
                * (-560.0 * cv * cu6 * su2 + 80.0 * cv * cu8 - 288.0 * cv * cu5 * su
                    + 400.0 * cv * cu4 * su2
                    - 80.0 * cv * cu6
                    + 192.0 * cv * cu3 * su
                    + 15.0 * su2 * cv * cu2
                    - 5.0 * cv * cu4
                    + 6.0 * su * cv * cu
                    + 5.0 * cv * cu2
                    - 5.0 * su2 * cv
                    - 60.0 * cu),
        2.0 / 15.0 * sv * (5.0 * cu2 - 5.0 * su2),
    ];

    let dv = [
        -2.0 / 15.0 * cu * (-3.0 * sv - 5.0 * su * sv * cu),
        -1.0 / 15.0
            * su
            * (-80.0 * sv * cu7 * su - 48.0 * sv * cu6
                + 80.0 * sv * cu5 * su
                + 48.0 * sv * cu4
                + 5.0 * sv * cu3 * su
                + 3.0 * sv * cu2
                - 5.0 * su * sv * cu
                - 3.0 * sv),
        2.0 / 15.0 * cv * sub_z,
    ];

    (pt, du, dv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_klein_bottle() {
        let pd = klein_bottle(&KleinBottleParams::default());
        assert_eq!(pd.points.len(), (32 + 1) * 16);
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
        assert_eq!(pd.points.len(), (3 + 1) * 3);
        assert_eq!(pd.polys.num_cells(), 9);
    }

    #[test]
    fn evaluate_matches_vtk_reference_point() {
        let (pt, du, dv) = evaluate_klein(0.0, 0.0);
        assert!((pt[0] + 0.4).abs() < 1e-12);
        assert!(pt[1].abs() < 1e-12);
        assert!(pt[2].abs() < 1e-12);
        assert!(du.iter().all(|x| x.is_finite()));
        assert!(dv.iter().all(|x| x.is_finite()));
    }
}
