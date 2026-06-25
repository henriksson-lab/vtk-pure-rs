use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Shrink each cell toward its centroid.
///
/// Each output cell gets its own copy of its vertices, so the output has no
/// shared vertices between cells. Polylines are split into shrunk line
/// segments, and triangle strips are split into triangle polygons.
pub fn shrink(input: &PolyData, factor: f64) -> PolyData {
    let factor = factor.clamp(0.0, 1.0);
    let pts = input.points.as_flat_slice();
    let total_out_pts = input.verts.connectivity_len()
        + input
            .lines
            .iter()
            .map(|cell| cell.len().saturating_sub(1) * 2)
            .sum::<usize>()
        + input.polys.connectivity_len()
        + input
            .strips
            .iter()
            .map(|cell| cell.len().saturating_sub(2) * 3)
            .sum::<usize>();
    let mut out_flat = Vec::with_capacity(total_out_pts * 3);
    let mut old_point_ids = Vec::with_capacity(total_out_pts);

    let mut pd = PolyData::new();
    pd.verts = copy_vertices(&input.verts, pts, &mut out_flat, &mut old_point_ids);
    pd.lines = shrink_lines(&input.lines, pts, factor, &mut out_flat, &mut old_point_ids);
    pd.polys = shrink_polys(&input.polys, pts, factor, &mut out_flat, &mut old_point_ids);
    append_shrunk_strips_to_polys(
        &input.strips,
        pts,
        factor,
        &mut out_flat,
        &mut old_point_ids,
        &mut pd.polys,
    );
    pd.points = Points::from_flat_vec(out_flat);
    *pd.point_data_mut() = copy_point_data(input.point_data(), &old_point_ids);
    *pd.cell_data_mut() = input.cell_data().clone();
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

fn copy_vertices(
    cells: &CellArray,
    pts: &[f64],
    out_flat: &mut Vec<f64>,
    old_point_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells.iter() {
        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let new_id = copy_point(pts, pid as usize, out_flat, old_point_ids);
            new_ids.push(new_id);
        }
        out.push_cell(&new_ids);
    }
    out
}

fn shrink_lines(
    cells: &CellArray,
    pts: &[f64],
    factor: f64,
    out_flat: &mut Vec<f64>,
    old_point_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells.iter() {
        for segment in cell.windows(2) {
            let p1 = point(pts, segment[0] as usize);
            let p2 = point(pts, segment[1] as usize);
            let center = [
                (p1[0] + p2[0]) / 2.0,
                (p1[1] + p2[1]) / 2.0,
                (p1[2] + p2[2]) / 2.0,
            ];
            let id1 = push_shrunk_point(
                p1,
                center,
                factor,
                segment[0] as usize,
                out_flat,
                old_point_ids,
            );
            let id2 = push_shrunk_point(
                p2,
                center,
                factor,
                segment[1] as usize,
                out_flat,
                old_point_ids,
            );
            out.push_cell(&[id1, id2]);
        }
    }
    out
}

fn shrink_polys(
    cells: &CellArray,
    pts: &[f64],
    factor: f64,
    out_flat: &mut Vec<f64>,
    old_point_ids: &mut Vec<usize>,
) -> CellArray {
    let offsets = cells.offsets();
    let conn = cells.connectivity();
    let mut out_off = Vec::with_capacity(cells.num_cells() + 1);
    let mut out_conn = Vec::with_capacity(conn.len());
    out_off.push(0);

    for ci in 0..cells.num_cells() {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let n = (end - start) as f64;
        if n == 0.0 {
            out_off.push(out_conn.len() as i64);
            continue;
        }

        let mut center = [0.0; 3];
        for &pid in &conn[start..end] {
            let base = pid as usize * 3;
            center[0] += pts[base];
            center[1] += pts[base + 1];
            center[2] += pts[base + 2];
        }
        center[0] /= n;
        center[1] /= n;
        center[2] /= n;

        for &pid in &conn[start..end] {
            let base = pid as usize * 3;
            let new_id = (out_flat.len() / 3) as i64;
            out_flat.push(center[0] + factor * (pts[base] - center[0]));
            out_flat.push(center[1] + factor * (pts[base + 1] - center[1]));
            out_flat.push(center[2] + factor * (pts[base + 2] - center[2]));
            out_conn.push(new_id);
            old_point_ids.push(pid as usize);
        }
        out_off.push(out_conn.len() as i64);
    }

    CellArray::from_raw(out_off, out_conn)
}

fn append_shrunk_strips_to_polys(
    strips: &CellArray,
    pts: &[f64],
    factor: f64,
    out_flat: &mut Vec<f64>,
    old_point_ids: &mut Vec<usize>,
    out_polys: &mut CellArray,
) {
    for cell in strips.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 0..cell.len() - 2 {
            let p1 = point(pts, cell[i] as usize);
            let p2 = point(pts, cell[i + 1] as usize);
            let p3 = point(pts, cell[i + 2] as usize);
            let center = [
                (p1[0] + p2[0] + p3[0]) / 3.0,
                (p1[1] + p2[1] + p3[1]) / 3.0,
                (p1[2] + p2[2] + p3[2]) / 3.0,
            ];

            let id1 = push_shrunk_point(
                p1,
                center,
                factor,
                cell[i] as usize,
                out_flat,
                old_point_ids,
            );
            let id2 = push_shrunk_point(
                p2,
                center,
                factor,
                cell[i + 1] as usize,
                out_flat,
                old_point_ids,
            );
            let id3 = push_shrunk_point(
                p3,
                center,
                factor,
                cell[i + 2] as usize,
                out_flat,
                old_point_ids,
            );

            if i % 2 == 0 {
                out_polys.push_cell(&[id1, id2, id3]);
            } else {
                out_polys.push_cell(&[id2, id1, id3]);
            }
        }
    }
}

fn point(pts: &[f64], point_id: usize) -> [f64; 3] {
    let base = point_id * 3;
    [pts[base], pts[base + 1], pts[base + 2]]
}

fn copy_point(
    pts: &[f64],
    old_id: usize,
    out_flat: &mut Vec<f64>,
    old_point_ids: &mut Vec<usize>,
) -> i64 {
    let p = point(pts, old_id);
    let new_id = (out_flat.len() / 3) as i64;
    out_flat.extend_from_slice(&p);
    old_point_ids.push(old_id);
    new_id
}

fn push_shrunk_point(
    p: [f64; 3],
    center: [f64; 3],
    factor: f64,
    old_id: usize,
    out_flat: &mut Vec<f64>,
    old_point_ids: &mut Vec<usize>,
) -> i64 {
    let new_id = (out_flat.len() / 3) as i64;
    out_flat.push(center[0] + factor * (p[0] - center[0]));
    out_flat.push(center[1] + factor * (p[1] - center[1]));
    out_flat.push(center[2] + factor * (p[2] - center[2]));
    old_point_ids.push(old_id);
    new_id
}

fn copy_point_data(input: &DataSetAttributes, old_point_ids: &[usize]) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();
    for ai in 0..input.num_arrays() {
        let Some(array) = input.get_array_by_index(ai) else {
            continue;
        };
        output.add_array(copy_array_tuples(array, old_point_ids));
    }
    if let Some(a) = input.scalars() {
        output.set_active_scalars(a.name());
    }
    if let Some(a) = input.vectors() {
        output.set_active_vectors(a.name());
    }
    if let Some(a) = input.normals() {
        output.set_active_normals(a.name());
    }
    if let Some(a) = input.tcoords() {
        output.set_active_tcoords(a.name());
    }
    if let Some(a) = input.tensors() {
        output.set_active_tensors(a.name());
    }
    output
}

fn copy_array_tuples(array: &AnyDataArray, old_point_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($arr:expr, $variant:ident) => {{
            let mut out = DataArray::new($arr.name(), $arr.num_components());
            for &old_id in old_point_ids {
                out.push_tuple($arr.tuple(old_id));
            }
            AnyDataArray::$variant(out)
        }};
    }
    match array {
        AnyDataArray::F32(a) => copy!(a, F32),
        AnyDataArray::F64(a) => copy!(a, F64),
        AnyDataArray::I8(a) => copy!(a, I8),
        AnyDataArray::I16(a) => copy!(a, I16),
        AnyDataArray::I32(a) => copy!(a, I32),
        AnyDataArray::I64(a) => copy!(a, I64),
        AnyDataArray::U8(a) => copy!(a, U8),
        AnyDataArray::U16(a) => copy!(a, U16),
        AnyDataArray::U32(a) => copy!(a, U32),
        AnyDataArray::U64(a) => copy!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrink_factor_one_preserves_geometry() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = shrink(&pd, 1.0);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        let p0 = result.points.get(0);
        assert!(p0[0].abs() < 1e-10);
    }

    #[test]
    fn shrink_factor_zero_collapses_to_centroid() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = shrink(&pd, 0.0);
        assert_eq!(result.points.len(), 3);
        for i in 0..3 {
            let p = result.points.get(i);
            assert!((p[0] - 1.0).abs() < 1e-10);
            assert!((p[1] - 1.0).abs() < 1e-10);
            assert!(p[2].abs() < 1e-10);
        }
    }

    #[test]
    fn shrink_duplicates_shared_points() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = shrink(&pd, 0.5);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn shrink_uses_vtk_polydata_topology() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = shrink(&pd, 0.5);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.strips.num_cells(), 0);
        assert_eq!(result.points.len(), 8);
    }

    #[test]
    fn shrink_splits_polyline_segments() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = shrink(&pd, 1.0);
        assert_eq!(result.lines.num_cells(), 2);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.lines.cell(0), &[0, 1]);
        assert_eq!(result.lines.cell(1), &[2, 3]);
    }

    #[test]
    fn shrink_converts_strips_to_polys() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = shrink(&pd, 1.0);
        assert_eq!(result.strips.num_cells(), 0);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[4, 3, 5]);
    }

    #[test]
    fn shrink_factor_is_clamped() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let below = shrink(&pd, -1.0);
        let p = below.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);

        let above = shrink(&pd, 2.0);
        assert_eq!(above.points.get(0), [0.0, 0.0, 0.0]);
    }
}
