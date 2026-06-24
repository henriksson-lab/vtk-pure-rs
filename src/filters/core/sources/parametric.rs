use crate::data::{CellArray, DataArray, Points, PolyData};

/// Generate a parametric surface from a function `f(u, v) -> [x, y, z]`.
///
/// The surface is sampled on a `u_resolution × v_resolution` grid over
/// the parameter domain `[u_min, u_max] × [v_min, v_max]`.
pub fn parametric_function<F>(
    f: F,
    u_range: [f64; 2],
    v_range: [f64; 2],
    u_resolution: usize,
    v_resolution: usize,
) -> PolyData
where
    F: Fn(f64, f64) -> [f64; 3],
{
    let nu = u_resolution.max(2);
    let nv = v_resolution.max(2);

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("TCoords", 2);
    let mut polys = CellArray::new();

    let du = (u_range[1] - u_range[0]) / (nu - 1) as f64;
    let dv = (v_range[1] - v_range[0]) / (nv - 1) as f64;
    let eps = 1e-6;

    for j in 0..nv {
        let v = v_range[0] + j as f64 * dv;
        for i in 0..nu {
            let u = u_range[0] + i as f64 * du;
            let p = f(u, v);
            points.push(p);
            tcoords.push_tuple(&[i as f64 / (nu - 1) as f64, j as f64 / (nv - 1) as f64]);

            // Numerical normal via cross product of partial derivatives
            let pu = f(u + eps, v);
            let pv = f(u, v + eps);
            let du_vec = [pu[0] - p[0], pu[1] - p[1], pu[2] - p[2]];
            let dv_vec = [pv[0] - p[0], pv[1] - p[1], pv[2] - p[2]];
            let n = cross(du_vec, dv_vec);
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-20 {
                normals.push_tuple(&[n[0] / len, n[1] / len, n[2] / len]);
            } else {
                normals.push_tuple(&[0.0, 0.0, 1.0]);
            }
        }
    }

    // Generate quads
    for j in 0..nv - 1 {
        for i in 0..nu - 1 {
            let p0 = (j * nu + i) as i64;
            let p1 = p0 + 1;
            let p2 = p0 + nu as i64 + 1;
            let p3 = p0 + nu as i64;
            polys.push_cell(&[p0, p1, p2, p3]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(tcoords.into());
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Generate a torus as a parametric surface.
pub fn torus(major_radius: f64, minor_radius: f64, resolution: usize) -> PolyData {
    let pi2 = 2.0 * std::f64::consts::PI;
    parametric_function(
        |u, v| {
            let r = major_radius + minor_radius * v.cos();
            [r * u.cos(), r * u.sin(), minor_radius * v.sin()]
        },
        [0.0, pi2],
        [0.0, pi2],
        resolution,
        resolution,
    )
}

/// Generate a Klein bottle as a parametric surface.
pub fn klein_bottle(resolution: usize) -> PolyData {
    let pi2 = 2.0 * std::f64::consts::PI;
    parametric_function(
        |u, v| {
            let cu = u.cos();
            let su = u.sin();
            let cv = v.cos();
            let sv = v.sin();
            let r = 4.0 * (1.0 - cu / 2.0);
            if u < std::f64::consts::PI {
                [
                    6.0 * cu * (1.0 + su) + r * cu * cv,
                    16.0 * su + r * su * cv,
                    r * sv,
                ]
            } else {
                [6.0 * cu * (1.0 + su) - r * cv, 16.0 * su, r * sv]
            }
        },
        [0.0, pi2],
        [0.0, pi2],
        resolution,
        resolution,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parametric_plane() {
        let pd = parametric_function(|u, v| [u, v, 0.0], [0.0, 1.0], [0.0, 1.0], 4, 4);
        assert_eq!(pd.points.len(), 16); // 4x4
        assert_eq!(pd.polys.num_cells(), 9); // 3x3 quads
    }

    #[test]
    fn torus_surface() {
        let pd = torus(1.0, 0.3, 16);
        assert_eq!(pd.points.len(), 256); // 16x16
        assert_eq!(pd.polys.num_cells(), 225); // 15x15 quads
    }

    #[test]
    fn klein_bottle_surface() {
        let pd = klein_bottle(10);
        assert_eq!(pd.points.len(), 100);
        assert!(pd.polys.num_cells() > 0);
    }
}
