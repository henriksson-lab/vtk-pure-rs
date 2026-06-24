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

        // Find edge crossings using flat point access
        let mut c0x = 0.0f64;
        let mut c0y = 0.0f64;
        let mut c0z = 0.0f64;
        let mut c1x = 0.0f64;
        let mut c1y = 0.0f64;
        let mut c1z = 0.0f64;
        let mut num_crossings = 0u32;

        for i in 0..n {
            let j = if i + 1 < n { i + 1 } else { 0 };
            let ai = cell[i] as usize;
            let aj = cell[j] as usize;
            let di = unsafe { *dists.get_unchecked(ai) };
            let dj = unsafe { *dists.get_unchecked(aj) };

            if (di >= 0.0) != (dj >= 0.0) {
                let t = di / (di - dj);
                let bi = ai * 3;
                let bj = aj * 3;
                let px = pts[bi] + t * (pts[bj] - pts[bi]);
                let py = pts[bi + 1] + t * (pts[bj + 1] - pts[bi + 1]);
                let pz = pts[bi + 2] + t * (pts[bj + 2] - pts[bi + 2]);
                if num_crossings == 0 {
                    c0x = px;
                    c0y = py;
                    c0z = pz;
                    num_crossings = 1;
                } else if num_crossings == 1 {
                    c1x = px;
                    c1y = py;
                    c1z = pz;
                    num_crossings = 2;
                }
            } else if di.abs() < 1e-10 && dj.abs() >= 1e-10 {
                let bi = ai * 3;
                if num_crossings == 0 {
                    c0x = pts[bi];
                    c0y = pts[bi + 1];
                    c0z = pts[bi + 2];
                    num_crossings = 1;
                } else if num_crossings == 1 {
                    c1x = pts[bi];
                    c1y = pts[bi + 1];
                    c1z = pts[bi + 2];
                    num_crossings = 2;
                }
            }
        }

        if num_crossings == 2 {
            let idx = (pts_flat.len() / 3) as i64;
            pts_flat.push(c0x);
            pts_flat.push(c0y);
            pts_flat.push(c0z);
            pts_flat.push(c1x);
            pts_flat.push(c1y);
            pts_flat.push(c1z);
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
}
