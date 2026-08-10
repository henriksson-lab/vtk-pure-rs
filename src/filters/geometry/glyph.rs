use crate::data::{CellArray, Points, PolyData};

/// Place a copy of a glyph mesh at each point of the input PolyData.
///
/// The glyph is translated to each input point. If `scale_by_scalar` is true
/// and active scalars exist, each glyph is uniformly scaled by the scalar value.
pub fn glyph(
    input: &PolyData,
    glyph_source: &PolyData,
    scale_factor: f64,
    scale_by_scalar: bool,
) -> PolyData {
    let n = input.points.len();
    let glyph_n_pts = glyph_source.points.len();

    if n == 0 || glyph_n_pts == 0 {
        return PolyData::new();
    }

    let scalars = scale_by_scalar
        .then(|| input.point_data().scalars())
        .flatten();
    let input_points = input.points.as_flat_slice();
    let glyph_points = glyph_source.points.as_flat_slice();
    let mut point_coords = Vec::with_capacity(n * glyph_n_pts * 3);

    let mut scalar_buf = [0.0f64];
    for (i, center) in input_points.chunks_exact(3).enumerate() {
        let scale = if let Some(scalars) = scalars {
            scalars.tuple_as_f64(i, &mut scalar_buf);
            scalar_buf[0] * scale_factor
        } else {
            scale_factor
        };

        for gp in glyph_points.chunks_exact(3) {
            point_coords.push(center[0] + gp[0] * scale);
            point_coords.push(center[1] + gp[1] * scale);
            point_coords.push(center[2] + gp[2] * scale);
        }
    }

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(point_coords);
    pd.verts = repeat_cells_with_point_offset(&glyph_source.verts, n, glyph_n_pts);
    pd.lines = repeat_cells_with_point_offset(&glyph_source.lines, n, glyph_n_pts);
    pd.polys = repeat_cells_with_point_offset(&glyph_source.polys, n, glyph_n_pts);
    pd.strips = repeat_cells_with_point_offset(&glyph_source.strips, n, glyph_n_pts);
    pd
}

fn repeat_cells_with_point_offset(
    src: &CellArray,
    copies: usize,
    points_per_copy: usize,
) -> CellArray {
    let src_num_cells = src.num_cells();
    let src_conn = src.connectivity();
    if src_num_cells == 0 || copies == 0 {
        return CellArray::new();
    }

    let src_offsets = src.offsets();
    let src_conn_len = src_conn.len();
    let mut offsets = Vec::with_capacity(copies * src_num_cells + 1);
    let mut connectivity = Vec::with_capacity(copies * src_conn_len);
    offsets.push(0);

    for copy in 0..copies {
        let point_offset = (copy * points_per_copy) as i64;
        let conn_offset = (copy * src_conn_len) as i64;

        offsets.extend(src_offsets[1..].iter().map(|&offset| offset + conn_offset));
        connectivity.extend(src_conn.iter().map(|&point_id| point_id + point_offset));
    }

    CellArray::from_raw(offsets, connectivity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_at_two_points() {
        // Input: two points
        let mut input = PolyData::new();
        input.points.push([0.0, 0.0, 0.0]);
        input.points.push([5.0, 0.0, 0.0]);

        // Glyph: single triangle
        let glyph_src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = glyph(&input, &glyph_src, 1.0, false);
        assert_eq!(result.points.len(), 6); // 3 per glyph * 2 points
        assert_eq!(result.polys.num_cells(), 2);

        // First glyph at origin
        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
        // Second glyph at (5, 0, 0)
        assert_eq!(result.points.get(3), [5.0, 0.0, 0.0]);
        assert_eq!(result.points.get(4), [6.0, 0.0, 0.0]);
    }

    #[test]
    fn glyph_with_scaling() {
        let mut input = PolyData::new();
        input.points.push([0.0, 0.0, 0.0]);

        let glyph_src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = glyph(&input, &glyph_src, 2.0, false);
        // Glyph scaled by 2
        assert_eq!(result.points.get(1), [2.0, 0.0, 0.0]);
    }

    #[test]
    fn glyph_repeats_all_cell_arrays() {
        let mut input = PolyData::new();
        input.points.push([0.0, 0.0, 0.0]);
        input.points.push([10.0, 0.0, 0.0]);

        let mut glyph_src = PolyData::new();
        glyph_src.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]);
        glyph_src.verts.push_cell(&[0]);
        glyph_src.lines.push_cell(&[0, 1]);
        glyph_src.polys.push_cell(&[0, 1, 2]);
        glyph_src.strips.push_cell(&[0, 1, 2, 3]);

        let result = glyph(&input, &glyph_src, 1.0, false);

        assert_eq!(result.points.len(), 8);
        assert_eq!(result.verts.num_cells(), 2);
        assert_eq!(result.lines.num_cells(), 2);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.strips.num_cells(), 2);
        assert_eq!(result.lines.cell(1), &[4, 5]);
        assert_eq!(result.polys.cell(1), &[4, 5, 6]);
        assert_eq!(result.strips.cell(1), &[4, 5, 6, 7]);
    }
}
