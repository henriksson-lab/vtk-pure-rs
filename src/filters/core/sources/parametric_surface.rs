//! Generic parametric surface from (u,v) -> (x,y,z) function.

use crate::data::{CellArray, Points, PolyData};
use std::f64::consts::PI;

/// Create a parametric surface from a function f(u, v) -> [x, y, z].
pub fn parametric_surface(
    u_range: [f64; 2],
    v_range: [f64; 2],
    u_res: usize,
    v_res: usize,
    f: impl Fn(f64, f64) -> [f64; 3],
) -> PolyData {
    let pts_u = u_res.max(2);
    let pts_v = v_res.max(2);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    let u_step = (u_range[1] - u_range[0]) / (pts_u - 1) as f64;
    let v_step = (v_range[1] - v_range[0]) / (pts_v - 1) as f64;

    for i in 0..pts_u {
        let u = u_range[0] + i as f64 * u_step;
        for j in 0..pts_v {
            let v = v_range[0] + j as f64 * v_step;
            points.push(f(u, v));
        }
    }

    make_triangles(&mut polys, pts_u, pts_v, false, false, false, false, true);

    let mut result = PolyData::new();
    result.points = points;
    result.polys = polys;
    result
}

/// Create a closed parametric surface (wraps in both u and v).
pub fn parametric_surface_closed(
    u_res: usize,
    v_res: usize,
    f: impl Fn(f64, f64) -> [f64; 3],
) -> PolyData {
    let pts_u = u_res.max(3);
    let pts_v = v_res.max(3);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let pi2 = 2.0 * PI;

    for i in 0..pts_u {
        let u = pi2 * i as f64 / (pts_u - 1) as f64;
        for j in 0..pts_v {
            let v = pi2 * j as f64 / (pts_v - 1) as f64;
            points.push(f(u, v));
        }
    }

    make_triangles(&mut polys, pts_u, pts_v, true, true, false, false, true);

    let mut result = PolyData::new();
    result.points = points;
    result.polys = polys;
    result
}

/// Example: Klein bottle using VTK's `vtkParametricKlein` parametrization.
pub fn klein_bottle(r: f64, res: usize) -> PolyData {
    let pts_u = res.max(3);
    let pts_v = res.max(3);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..pts_u {
        let u = PI * i as f64 / (pts_u - 1) as f64;
        for j in 0..pts_v {
            let v = 2.0 * PI * j as f64 / (pts_v - 1) as f64;
            let (pt, _, _) = crate::filters::core::sources::klein_bottle::evaluate_klein(u, v);
            points.push([r * pt[0], r * pt[1], r * pt[2]]);
        }
    }

    make_triangles(&mut polys, pts_u, pts_v, false, true, false, false, false);

    let mut result = PolyData::new();
    result.points = points;
    result.polys = polys;
    result
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
            let (id2, new_id4) = if twist_u {
                let id2 = pts_v - 1 - j;
                (id2, id2 - 1)
            } else {
                let id2 = j;
                (id2, id2 + 1)
            };
            id4 = new_id4;
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
    fn test_paraboloid() {
        let s = parametric_surface([-1.0, 1.0], [-1.0, 1.0], 10, 10, |u, v| {
            [u, v, u * u + v * v]
        });
        assert_eq!(s.points.len(), 100);
        assert_eq!(s.polys.num_cells(), 162);
    }
    #[test]
    fn test_closed() {
        let s = parametric_surface_closed(12, 24, |u, v| {
            let r = 2.0 + 0.5 * u.cos();
            [r * v.cos(), r * v.sin(), 0.5 * u.sin()]
        });
        assert_eq!(s.points.len(), 288);
        assert_eq!(s.polys.num_cells(), 576);
    }
    #[test]
    fn test_klein() {
        let k = klein_bottle(3.0, 16);
        assert_eq!(k.points.len(), 256);
    }
}
