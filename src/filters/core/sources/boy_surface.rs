use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating Boy's surface (an immersion of the real projective plane in 3D).
pub struct BoySurfaceParams {
    pub center: [f64; 3],
    pub radius: f64,
    pub resolution: usize,
    pub z_scale: f64,
}

impl Default for BoySurfaceParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
            resolution: 32,
            z_scale: 0.125,
        }
    }
}

/// Generate Boy's surface using VTK's `vtkParametricBoy` parametrization.
///
/// The parametrization maps (u, v) with u in [0, PI] and v in [0, PI] to 3D,
/// where the surface is an immersion of the real projective plane.
pub fn boy_surface(params: &BoySurfaceParams) -> PolyData {
    let n = params.resolution.max(3);
    let [cx, cy, cz] = params.center;
    let r = params.radius;

    let mut points = Points::new();
    let mut normals_data = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    for j in 0..=n {
        let v = PI * j as f64 / n as f64;
        for i in 0..=n {
            let u = PI * i as f64 / n as f64;

            let (point, du, dv) = evaluate_boy(u, v, params.z_scale);
            let [x, y, z] = point;

            points.push([cx + r * x, cy + r * y, cz + r * z]);

            let nx = du[1] * dv[2] - du[2] * dv[1];
            let ny = du[2] * dv[0] - du[0] * dv[2];
            let nz = du[0] * dv[1] - du[1] * dv[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-12);
            normals_data.push_tuple(&[nx / len, ny / len, nz / len]);
        }
    }

    // Create quads connecting the grid
    let cols = n + 1;
    for j in 0..n {
        for i in 0..n {
            let p0 = (j * cols + i) as i64;
            let p1 = (j * cols + i + 1) as i64;
            let p2 = ((j + 1) * cols + i + 1) as i64;
            let p3 = ((j + 1) * cols + i) as i64;
            polys.push_cell(&[p0, p1, p2, p3]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals_data.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

/// VTK `vtkParametricBoy::Evaluate`, translated with Rust names.
pub(crate) fn evaluate_boy(u: f64, v: f64, z_scale: f64) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let cu = u.cos();
    let su = u.sin();
    let sv = v.sin();

    let x = cu * sv;
    let y = su * sv;
    let z = v.cos();

    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x3 * x;
    let y2 = y * y;
    let y3 = y2 * y;
    let y4 = y3 * y;
    let z2 = z * z;
    let z3 = z2 * z;
    let z4 = z3 * z;

    let sr3 = 3.0_f64.sqrt();

    let point = [
        1.0 / 2.0
            * (2.0 * x2 - y2 - z2
                + 2.0 * y * z * (y2 - z2)
                + z * x * (x2 - z2)
                + x * y * (y2 - x2)),
        sr3 / 2.0 * (y2 - z2 + (z * x * (z2 - x2) + x * y * (y2 - x2))),
        z_scale
            * (x + y + z)
            * ((x + y + z) * (x + y + z) * (x + y + z) + 4.0 * (y - x) * (z - y) * (x - z)),
    ];

    let du = [
        -1.0 / 2.0 * x4 - z3 * x + 3.0 * y2 * x2 - 3.0 / 2.0 * z * x2 * y + 3.0 * z * x * y2
            - 3.0 * y * x
            - 1.0 / 2.0 * y4
            + 1.0 / 2.0 * z3 * y,
        -1.0 / 2.0 * sr3 * x4 + 3.0 * sr3 * y2 * x2 + 3.0 / 2.0 * sr3 * z * x2 * y + sr3 * y * x
            - 1.0 / 2.0 * sr3 * y4
            - 1.0 / 2.0 * sr3 * z3 * y,
        x4 + 3.0 / 2.0 * z * x3 + 3.0 / 2.0 * z2 * x2 + x3 * y - 3.0 * x2 * y2 + 3.0 * z * x2 * y
            - y3 * x
            - 3.0 / 2.0 * z * y3
            - 3.0 / 2.0 * z2 * y2
            - z3 * y,
    ];

    let dv = [
        (3.0 / 2.0 * z2 * x2 + 2.0 * z * x - 1.0 / 2.0 * z4) * cu
            + (-2.0 * z * x3 + 2.0 * z * x * y2 + 3.0 * z2 * y2 - z * y - z4) * su
            + (-1.0 / 2.0 * x3 + 3.0 / 2.0 * z2 * x - y3 + 3.0 * z2 * y + z) * sv,
        (-3.0 / 2.0 * sr3 * z2 * x2 + 1.0 / 2.0 * sr3 * z4) * cu
            + (-2.0 * sr3 * z * x3 + 2.0 * sr3 * z * y2 * x + sr3 * z * y) * su
            + (1.0 / 2.0 * sr3 * x3 - 3.0 / 2.0 * sr3 * z2 * x + sr3 * z) * sv,
        (1.0 / 2.0 * z * x3 + 3.0 / 2.0 * z3 * x + z4) * cu
            + (4.0 * z * x3
                + 3.0 * z * x2 * y
                + 9.0 / 2.0 * z2 * x2
                + 9.0 / 2.0 * z2 * x * y
                + 3.0 * z3 * x
                + 1.0 / 2.0 * z * y3
                + 3.0 * z2 * y2
                + 3.0 / 2.0 * z3 * y)
                * su
            + (-3.0 / 2.0 * x2 * y
                - 3.0 / 2.0 * z * x2
                - 3.0 / 2.0 * x * y2
                - 3.0 * z * x * y
                - 3.0 * z2 * x
                - y3
                - 3.0 / 2.0 * z * y2
                - 1.0 / 2.0 * z3)
                * sv,
    ];

    (point, du, dv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_boy_surface() {
        let pd = boy_surface(&BoySurfaceParams::default());
        let n = 32usize;
        assert_eq!(pd.points.len(), (n + 1) * (n + 1));
        assert_eq!(pd.polys.num_cells(), n * n);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn minimal_boy_surface() {
        let pd = boy_surface(&BoySurfaceParams {
            resolution: 3,
            ..Default::default()
        });
        assert!(pd.points.len() > 0);
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn custom_center_and_radius() {
        let pd = boy_surface(&BoySurfaceParams {
            center: [1.0, 2.0, 3.0],
            radius: 2.0,
            resolution: 4,
            ..Default::default()
        });
        assert!(pd.points.len() > 0);
    }
}
