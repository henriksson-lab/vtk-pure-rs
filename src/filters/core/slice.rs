use crate::data::{CellArray, Points, PolyData};

/// Slice a PolyData mesh with a plane, producing intersection line segments.
///
/// Returns a PolyData containing line cells where the mesh intersects the plane.
/// The plane is defined by a point on the plane and the plane normal.
pub fn slice_by_plane(input: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let nc = input.polys.num_cells();
    if nc == 0 {
        return PolyData::new();
    }

    if normal[0] != 0.0 && normal[1] == 0.0 && normal[2] == 0.0 {
        return slice_by_x_plane(input, origin[0], normal[0]);
    }

    // Pre-compute signed distances using flat slice access.
    // Edge interpolation also uses flat pts[] indexing to avoid per-point get() overhead.
    let np = input.points.len();
    let pts = input.points.as_flat_slice();
    let (nx, ny, nz) = (normal[0], normal[1], normal[2]);
    let (ox, oy, oz) = (origin[0], origin[1], origin[2]);
    let mut dists = Vec::with_capacity(np);
    for i in 0..np {
        let b = i * 3;
        dists.push((pts[b] - ox) * nx + (pts[b + 1] - oy) * ny + (pts[b + 2] - oz) * nz);
    }

    // Pre-sized flat buffers for output
    let mut pts_flat: Vec<f64> = Vec::with_capacity(nc * 6);
    let mut line_conn: Vec<i64> = Vec::with_capacity(nc * 2);
    let mut line_off: Vec<i64> = Vec::with_capacity(nc + 1);
    line_off.push(0);

    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let cell = &conn[start..end];
        let n = cell.len();
        if n < 3 {
            continue;
        }

        if n == 3 {
            let mut crossings = [[0.0; 3]; 3];
            let mut num_crossings = 0usize;
            let mut valid_cell = true;

            for edge in 0..3 {
                let i = edge;
                let j = if edge + 1 < 3 { edge + 1 } else { 0 };
                if cell[i] < 0 || cell[j] < 0 {
                    valid_cell = false;
                    break;
                }
                let ai = cell[i] as usize;
                let aj = cell[j] as usize;
                if ai >= np || aj >= np {
                    valid_cell = false;
                    break;
                }

                let di = dists[ai];
                let dj = dists[aj];
                let bi = ai * 3;
                let bj = aj * 3;
                let pi = [pts[bi], pts[bi + 1], pts[bi + 2]];
                let pj = [pts[bj], pts[bj + 1], pts[bj + 2]];
                let on_i = di.abs() < 1e-10;
                let on_j = dj.abs() < 1e-10;

                if on_i && on_j {
                    push_unique_point_fixed(&mut crossings, &mut num_crossings, pi);
                    push_unique_point_fixed(&mut crossings, &mut num_crossings, pj);
                } else if on_i {
                    push_unique_point_fixed(&mut crossings, &mut num_crossings, pi);
                } else if on_j {
                    push_unique_point_fixed(&mut crossings, &mut num_crossings, pj);
                } else if (di > 0.0) != (dj > 0.0) {
                    let t = di / (di - dj);
                    push_unique_point_fixed(
                        &mut crossings,
                        &mut num_crossings,
                        [
                            pi[0] + t * (pj[0] - pi[0]),
                            pi[1] + t * (pj[1] - pi[1]),
                            pi[2] + t * (pj[2] - pi[2]),
                        ],
                    );
                }
            }

            if valid_cell && num_crossings == 2 {
                let idx = (pts_flat.len() / 3) as i64;
                pts_flat.extend_from_slice(&crossings[0]);
                pts_flat.extend_from_slice(&crossings[1]);
                line_conn.push(idx);
                line_conn.push(idx + 1);
                line_off.push(line_conn.len() as i64);
            }
            continue;
        }

        let mut crossings = Vec::<[f64; 3]>::with_capacity(4);
        let mut valid_cell = true;

        for i in 0..n {
            let j = if i + 1 < n { i + 1 } else { 0 };
            if cell[i] < 0 || cell[j] < 0 {
                valid_cell = false;
                break;
            }
            let ai = cell[i] as usize;
            let aj = cell[j] as usize;
            if ai >= np || aj >= np {
                valid_cell = false;
                break;
            }

            let di = dists[ai];
            let dj = dists[aj];
            let bi = ai * 3;
            let bj = aj * 3;
            let pi = [pts[bi], pts[bi + 1], pts[bi + 2]];
            let pj = [pts[bj], pts[bj + 1], pts[bj + 2]];
            let on_i = di.abs() < 1e-10;
            let on_j = dj.abs() < 1e-10;

            if on_i && on_j {
                push_unique_point(&mut crossings, pi);
                push_unique_point(&mut crossings, pj);
            } else if on_i {
                push_unique_point(&mut crossings, pi);
            } else if on_j {
                push_unique_point(&mut crossings, pj);
            } else if (di > 0.0) != (dj > 0.0) {
                let t = di / (di - dj);
                push_unique_point(
                    &mut crossings,
                    [
                        pi[0] + t * (pj[0] - pi[0]),
                        pi[1] + t * (pj[1] - pi[1]),
                        pi[2] + t * (pj[2] - pi[2]),
                    ],
                );
            }
        }

        if valid_cell && crossings.len() == 2 {
            let idx = (pts_flat.len() / 3) as i64;
            pts_flat.extend_from_slice(&crossings[0]);
            pts_flat.extend_from_slice(&crossings[1]);
            line_conn.push(idx);
            line_conn.push(idx + 1);
            line_off.push(line_conn.len() as i64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(pts_flat);
    pd.lines = CellArray::from_raw(line_off, line_conn);
    pd
}

fn slice_by_x_plane(input: &PolyData, x0: f64, nx: f64) -> PolyData {
    let nc = input.polys.num_cells();
    let np = input.points.len();
    let pts = input.points.as_flat_slice();
    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();

    let mut pts_flat: Vec<f64> = Vec::with_capacity(nc * 6);
    let mut line_conn: Vec<i64> = Vec::with_capacity(nc * 2);
    let mut line_off: Vec<i64> = Vec::with_capacity(nc + 1);
    line_off.push(0);

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let cell = &conn[start..end];
        if cell.len() != 3 {
            return slice_by_plane_generic(input, [x0, 0.0, 0.0], [nx, 0.0, 0.0]);
        }

        let mut ids = [0usize; 3];
        let mut valid_cell = true;
        for i in 0..3 {
            if cell[i] < 0 {
                valid_cell = false;
                break;
            }
            let id = cell[i] as usize;
            if id >= np {
                valid_cell = false;
                break;
            }
            ids[i] = id;
        }
        if !valid_cell {
            continue;
        }

        let d0 = (pts[ids[0] * 3] - x0) * nx;
        let d1 = (pts[ids[1] * 3] - x0) * nx;
        let d2 = (pts[ids[2] * 3] - x0) * nx;
        if (d0 > 0.0 && d1 > 0.0 && d2 > 0.0) || (d0 < 0.0 && d1 < 0.0 && d2 < 0.0) {
            continue;
        }

        let d = [d0, d1, d2];
        let mut crossings = [[0.0; 3]; 3];
        let mut num_crossings = 0usize;
        for edge in 0..3 {
            let i = edge;
            let j = if edge + 1 < 3 { edge + 1 } else { 0 };
            let di = d[i];
            let dj = d[j];
            let on_i = di.abs() < 1e-10;
            let on_j = dj.abs() < 1e-10;

            if on_i && on_j {
                push_unique_point_fixed(&mut crossings, &mut num_crossings, point_at(pts, ids[i]));
                push_unique_point_fixed(&mut crossings, &mut num_crossings, point_at(pts, ids[j]));
            } else if on_i {
                push_unique_point_fixed(&mut crossings, &mut num_crossings, point_at(pts, ids[i]));
            } else if on_j {
                push_unique_point_fixed(&mut crossings, &mut num_crossings, point_at(pts, ids[j]));
            } else if (di > 0.0) != (dj > 0.0) {
                let pi = point_at(pts, ids[i]);
                let pj = point_at(pts, ids[j]);
                let t = di / (di - dj);
                push_unique_point_fixed(
                    &mut crossings,
                    &mut num_crossings,
                    [
                        pi[0] + t * (pj[0] - pi[0]),
                        pi[1] + t * (pj[1] - pi[1]),
                        pi[2] + t * (pj[2] - pi[2]),
                    ],
                );
            }
        }

        if num_crossings == 2 {
            let idx = (pts_flat.len() / 3) as i64;
            pts_flat.extend_from_slice(&crossings[0]);
            pts_flat.extend_from_slice(&crossings[1]);
            line_conn.push(idx);
            line_conn.push(idx + 1);
            line_off.push(line_conn.len() as i64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(pts_flat);
    pd.lines = CellArray::from_raw(line_off, line_conn);
    pd
}

fn slice_by_plane_generic(input: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let nc = input.polys.num_cells();
    let np = input.points.len();
    let pts = input.points.as_flat_slice();
    let (nx, ny, nz) = (normal[0], normal[1], normal[2]);
    let (ox, oy, oz) = (origin[0], origin[1], origin[2]);
    let mut dists = Vec::with_capacity(np);
    for i in 0..np {
        let b = i * 3;
        dists.push((pts[b] - ox) * nx + (pts[b + 1] - oy) * ny + (pts[b + 2] - oz) * nz);
    }

    let mut pts_flat: Vec<f64> = Vec::with_capacity(nc * 6);
    let mut line_conn: Vec<i64> = Vec::with_capacity(nc * 2);
    let mut line_off: Vec<i64> = Vec::with_capacity(nc + 1);
    line_off.push(0);

    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let cell = &conn[start..end];
        let n = cell.len();
        if n < 3 {
            continue;
        }

        let mut crossings = Vec::<[f64; 3]>::with_capacity(4);
        let mut valid_cell = true;
        for i in 0..n {
            let j = if i + 1 < n { i + 1 } else { 0 };
            if cell[i] < 0 || cell[j] < 0 {
                valid_cell = false;
                break;
            }
            let ai = cell[i] as usize;
            let aj = cell[j] as usize;
            if ai >= np || aj >= np {
                valid_cell = false;
                break;
            }

            let di = dists[ai];
            let dj = dists[aj];
            let pi = point_at(pts, ai);
            let pj = point_at(pts, aj);
            let on_i = di.abs() < 1e-10;
            let on_j = dj.abs() < 1e-10;

            if on_i && on_j {
                push_unique_point(&mut crossings, pi);
                push_unique_point(&mut crossings, pj);
            } else if on_i {
                push_unique_point(&mut crossings, pi);
            } else if on_j {
                push_unique_point(&mut crossings, pj);
            } else if (di > 0.0) != (dj > 0.0) {
                let t = di / (di - dj);
                push_unique_point(
                    &mut crossings,
                    [
                        pi[0] + t * (pj[0] - pi[0]),
                        pi[1] + t * (pj[1] - pi[1]),
                        pi[2] + t * (pj[2] - pi[2]),
                    ],
                );
            }
        }

        if valid_cell && crossings.len() == 2 {
            let idx = (pts_flat.len() / 3) as i64;
            pts_flat.extend_from_slice(&crossings[0]);
            pts_flat.extend_from_slice(&crossings[1]);
            line_conn.push(idx);
            line_conn.push(idx + 1);
            line_off.push(line_conn.len() as i64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(pts_flat);
    pd.lines = CellArray::from_raw(line_off, line_conn);
    pd
}

#[inline(always)]
fn point_at(pts: &[f64], id: usize) -> [f64; 3] {
    let b = id * 3;
    [pts[b], pts[b + 1], pts[b + 2]]
}

fn push_unique_point(points: &mut Vec<[f64; 3]>, point: [f64; 3]) {
    if !points.iter().any(|p| {
        (p[0] - point[0]).abs() < 1e-10
            && (p[1] - point[1]).abs() < 1e-10
            && (p[2] - point[2]).abs() < 1e-10
    }) {
        points.push(point);
    }
}

fn push_unique_point_fixed(points: &mut [[f64; 3]; 3], len: &mut usize, point: [f64; 3]) {
    for existing in points.iter().take(*len) {
        if (existing[0] - point[0]).abs() < 1e-10
            && (existing[1] - point[1]).abs() < 1e-10
            && (existing[2] - point[2]).abs() < 1e-10
        {
            return;
        }
    }
    if *len < points.len() {
        points[*len] = point;
        *len += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_triangle_through_middle() {
        let pd = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        // Slice with plane at x=0, normal +X
        let result = slice_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn slice_misses_triangle() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        // Slice with plane at x=0 — triangle is entirely on positive side
        let result = slice_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn slice_multiple_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [0.0, -1.0, 1.0],
                [-1.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let result = slice_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.lines.num_cells(), 2);
    }

    #[test]
    fn slice_preserves_edge_on_plane() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = slice_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.points.get(0)[0], 0.0);
        assert_eq!(result.points.get(1)[0], 0.0);
    }
}
