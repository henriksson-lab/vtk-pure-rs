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

        let mut crossings = Vec::<[f64; 3]>::new();
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

fn push_unique_point(points: &mut Vec<[f64; 3]>, point: [f64; 3]) {
    if !points.iter().any(|p| {
        (p[0] - point[0]).abs() < 1e-10
            && (p[1] - point[1]).abs() < 1e-10
            && (p[2] - point[2]).abs() < 1e-10
    }) {
        points.push(point);
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
