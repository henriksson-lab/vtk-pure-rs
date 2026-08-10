use crate::data::PolyData;

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
/// Thin wrapper around
/// [`crate::filters::core::sources::parametric::klein_bottle`], which is the
/// single implementation of `vtkParametricKlein` +
/// `vtkParametricFunctionSource` (u in [0, PI], v in [0, 2*PI], no U join, a V
/// join, anti-clockwise ordering). The surface is then uniformly scaled by
/// `radius` and translated to `center`, neither of which VTK's parametric
/// function has; both leave the surface normals unchanged.
pub fn klein_bottle(params: &KleinBottleParams) -> PolyData {
    let mut pd = crate::filters::core::sources::parametric::klein_bottle_uv(
        params.u_resolution,
        params.v_resolution,
    );
    let [cx, cy, cz] = params.center;
    let r = params.radius;
    for i in 0..pd.points.len() {
        let p = pd.points.get(i);
        pd.points
            .set(i, [cx + r * p[0], cy + r * p[1], cz + r * p[2]]);
    }
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
        // vtkParametricFunctionSource samples UResolution x VResolution points.
        assert_eq!(pd.points.len(), 32 * 16);
        // JoinU = 0, JoinV = 1 -> (PtsU - 1) * PtsV quads, two triangles each.
        assert_eq!(pd.polys.num_cells(), (32 - 1) * 16 * 2);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn minimal_klein_bottle() {
        let pd = klein_bottle(&KleinBottleParams {
            u_resolution: 3,
            v_resolution: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 3 * 3);
        assert_eq!(pd.polys.num_cells(), (3 - 1) * 3 * 2);
    }

    #[test]
    fn center_and_radius_are_applied() {
        let pd = klein_bottle(&KleinBottleParams {
            center: [10.0, 20.0, 30.0],
            radius: 2.0,
            u_resolution: 8,
            v_resolution: 8,
        });
        let plain = crate::filters::core::sources::parametric::klein_bottle(8);
        for i in 0..pd.points.len() {
            let p = plain.points.get(i);
            let q = pd.points.get(i);
            assert!((q[0] - (10.0 + 2.0 * p[0])).abs() < 1e-12);
            assert!((q[1] - (20.0 + 2.0 * p[1])).abs() < 1e-12);
            assert!((q[2] - (30.0 + 2.0 * p[2])).abs() < 1e-12);
        }
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
