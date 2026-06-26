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
    parametric_function_with_topology(
        f,
        u_range,
        v_range,
        u_resolution,
        v_resolution,
        false,
        false,
        false,
        false,
        false,
    )
}

fn parametric_function_with_topology<F>(
    f: F,
    u_range: [f64; 2],
    v_range: [f64; 2],
    u_resolution: usize,
    v_resolution: usize,
    join_u: bool,
    join_v: bool,
    twist_u: bool,
    twist_v: bool,
    clockwise: bool,
) -> PolyData
where
    F: Fn(f64, f64) -> [f64; 3],
{
    let pts_u = u_resolution.max(2);
    let pts_v = v_resolution.max(2);

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("Textures", 2);
    let mut polys = CellArray::new();

    let du = (u_range[1] - u_range[0]) / (pts_u - 1) as f64;
    let dv = (v_range[1] - v_range[0]) / (pts_v - 1) as f64;
    let eps = 1e-6;

    for i in 0..pts_u {
        let u = u_range[0] + i as f64 * du;
        for j in 0..pts_v {
            let v = v_range[0] + j as f64 * dv;
            let p = f(u, v);
            points.push(p);
            tcoords.push_tuple(&[
                i as f64 / (pts_u - 1) as f64,
                1.0 - j as f64 / (pts_v - 1) as f64,
            ]);

            // Numerical normal via cross product of partial derivatives
            let pu = f(u + eps, v);
            let pv = f(u, v + eps);
            let du_vec = [pu[0] - p[0], pu[1] - p[1], pu[2] - p[2]];
            let dv_vec = [pv[0] - p[0], pv[1] - p[1], pv[2] - p[2]];
            let n = cross(dv_vec, du_vec);
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-20 {
                normals.push_tuple(&[n[0] / len, n[1] / len, n[2] / len]);
            } else {
                normals.push_tuple(&[0.0, 0.0, 1.0]);
            }
        }
    }

    make_triangles(
        &mut polys, pts_u, pts_v, join_u, join_v, twist_u, twist_v, clockwise,
    );

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(tcoords.into());
    pd.point_data_mut().set_active_tcoords("Textures");
    pd
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add_tri_cells(
    cell_array: &mut CellArray,
    id1: usize,
    id2: usize,
    id3: usize,
    id4: usize,
    clockwise: bool,
) {
    if clockwise {
        cell_array.push_cell(&[id1 as i64, id2 as i64, id3 as i64]);
        cell_array.push_cell(&[id1 as i64, id3 as i64, id4 as i64]);
    } else {
        cell_array.push_cell(&[id1 as i64, id3 as i64, id2 as i64]);
        cell_array.push_cell(&[id1 as i64, id4 as i64, id3 as i64]);
    }
}

fn make_triangles(
    cells: &mut CellArray,
    pts_u: usize,
    pts_v: usize,
    join_u: bool,
    join_v: bool,
    twist_u: bool,
    twist_v: bool,
    clockwise: bool,
) {
    let mut id3 = 0;
    let mut id4 = 0;

    for i in 0..pts_u - 1 {
        for j in 0..pts_v - 1 {
            let id1 = j + i * pts_v;
            let id2 = id1 + pts_v;
            id3 = id2 + 1;
            id4 = id1 + 1;
            add_tri_cells(cells, id1, id2, id3, id4, clockwise);
        }

        if join_v {
            let id1 = id4;
            let id2 = id3;
            let (id3, id4) = if twist_v {
                ((i + 1) * pts_v, i * pts_v)
            } else {
                (i * pts_v, (i + 1) * pts_v)
            };
            add_tri_cells(cells, id1, id2, id3, id4, clockwise);
        }
    }

    if join_u {
        for j in 0..pts_v - 1 {
            let id1 = j + (pts_u - 1) * pts_v;
            id3 = id1 + 1;
            let id2;
            if twist_u {
                id2 = pts_v - 1 - j;
                id4 = id2 - 1;
            } else {
                id2 = j;
                id4 = id2 + 1;
            }
            add_tri_cells(cells, id1, id2, id3, id4, clockwise);
        }

        if join_v {
            let id1 = id3;
            let id2 = id4;
            let (id3, id4) = if twist_u {
                if twist_v {
                    (pts_v - 1, (pts_u - 1) * pts_v)
                } else {
                    ((pts_u - 1) * pts_v, pts_v - 1)
                }
            } else if twist_v {
                (0, (pts_u - 1) * pts_v)
            } else {
                ((pts_u - 1) * pts_v, 0)
            };
            add_tri_cells(cells, id1, id2, id3, id4, clockwise);
        }
    }
}

/// Generate a torus as a parametric surface.
pub fn torus(major_radius: f64, minor_radius: f64, resolution: usize) -> PolyData {
    let pi2 = 2.0 * std::f64::consts::PI;
    parametric_function_with_topology(
        |u, v| {
            let r = major_radius + minor_radius * v.cos();
            [r * u.sin(), r * u.cos(), minor_radius * v.sin()]
        },
        [0.0, pi2],
        [0.0, pi2],
        resolution,
        resolution,
        true,
        true,
        false,
        false,
        false,
    )
}

/// Generate a Klein bottle as a parametric surface.
pub fn klein_bottle(resolution: usize) -> PolyData {
    let pi2 = 2.0 * std::f64::consts::PI;
    parametric_function_with_topology(
        |u, v| {
            let (pt, _, _) = crate::filters::core::sources::klein_bottle::evaluate_klein(u, v);
            pt
        },
        [0.0, std::f64::consts::PI],
        [0.0, pi2],
        resolution,
        resolution,
        false,
        true,
        false,
        false,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parametric_plane() {
        let pd = parametric_function(|u, v| [u, v, 0.0], [0.0, 1.0], [0.0, 1.0], 4, 4);
        assert_eq!(pd.points.len(), 16); // 4x4
        assert_eq!(pd.polys.num_cells(), 18); // 3x3 quads split into triangles
    }

    #[test]
    fn torus_surface() {
        let pd = torus(1.0, 0.3, 16);
        assert_eq!(pd.points.len(), 256); // 16x16
        assert_eq!(pd.polys.num_cells(), 512); // 16x16 joined quads split into triangles
    }

    #[test]
    fn klein_bottle_surface() {
        let pd = klein_bottle(10);
        assert_eq!(pd.points.len(), 100);
        assert!(pd.polys.num_cells() > 0);
    }
}
