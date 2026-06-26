//! Klein bottle immersed in 3D (figure-8 immersion).
use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

pub fn klein_bottle_figure8(scale: f64, u_res: usize, v_res: usize) -> PolyData {
    let pts_u = u_res.max(8);
    let pts_v = v_res.max(8);

    let maximum_u = PI + (2.0 * PI) / (pts_u - 1) as f64;
    let maximum_v = PI + (2.0 * PI) / (pts_v - 1) as f64;
    let u_step = (maximum_u + PI) / pts_u as f64;
    let v_step = (maximum_v + PI) / pts_v as f64;

    let mut points = Points::<f64>::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    let mut u = -PI - u_step;
    for _i in 0..pts_u {
        u += u_step;
        let mut v = -PI - v_step;
        for _j in 0..pts_v {
            v += v_step;
            let (pt, du, dv) = evaluate_figure8_klein(scale, u, v);

            points.push(pt);

            // vtkParametricFunctionSource uses Dv x Du for anti-clockwise ordering.
            normals.push_tuple(&[
                dv[1] * du[2] - dv[2] * du[1],
                dv[2] * du[0] - dv[0] * du[2],
                dv[0] * du[1] - dv[1] * du[0],
            ]);
        }
    }

    make_triangles(&mut polys, pts_u, pts_v);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

/// VTK `vtkParametricFigure8Klein::Evaluate`, translated with Rust names.
pub(crate) fn evaluate_figure8_klein(
    radius: f64,
    u: f64,
    v: f64,
) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let cu = u.cos();
    let cu2 = (u / 2.0).cos();
    let su = u.sin();
    let su2 = (u / 2.0).sin();
    let cv = v.cos();
    let c2v = (2.0 * v).cos();
    let s2v = (2.0 * v).sin();
    let sv = v.sin();
    let t = radius + sv * cu2 - s2v * su2 / 2.0;

    let pt = [cu * t, su * t, su2 * sv + cu2 * s2v / 2.0];

    let du = [
        -pt[1] - cu * (2.0 * sv * su2 + s2v * cu2) / 4.0,
        pt[0] - su * (2.0 * sv * su2 + s2v * cu2) / 4.0,
        cu2 * sv / 2.0 - su2 * s2v / 4.0,
    ];

    let dv = [
        cu * (cv * cu2 - c2v * su2),
        su * (cv * cu2 - c2v * su2),
        su2 * cv / 2.0 + cu2 * c2v,
    ];

    (pt, du, dv)
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

fn make_triangles(cells: &mut CellArray, pts_u: usize, pts_v: usize) {
    let join_u = true;
    let join_v = true;
    let twist_u = true;
    let twist_v = false;
    let clockwise = false;

    for i in 0..pts_u - 1 {
        let mut id3 = 0;
        let mut id4 = 0;

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
        let mut id3 = 0;
        let mut id4 = 0;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let k = klein_bottle_figure8(1.0, 16, 16);
        assert_eq!(k.points.len(), 256);
        assert_eq!(k.polys.num_cells(), 512);
        assert!(k.point_data().normals().is_some());
    }

    #[test]
    fn evaluate_matches_vtk_reference_point() {
        let (pt, du, dv) = evaluate_figure8_klein(1.0, 0.0, 0.0);
        assert!((pt[0] - 1.0).abs() < 1e-12);
        assert!(pt[1].abs() < 1e-12);
        assert!(pt[2].abs() < 1e-12);
        assert!(du.iter().all(|x| x.is_finite()));
        assert!(dv.iter().all(|x| x.is_finite()));
    }
}
