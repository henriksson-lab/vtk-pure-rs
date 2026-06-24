use crate::data::{CellArray, Points, PolyData};

/// Shrink each cell toward its centroid.
///
/// Each cell gets its own copy of its vertices, so the output has no shared
/// vertices between cells. The cell arrays and cell data are preserved.
pub fn shrink(input: &PolyData, factor: f64) -> PolyData {
    let pts = input.points.as_flat_slice();
    let total_out_pts = input.verts.connectivity_len()
        + input.lines.connectivity_len()
        + input.polys.connectivity_len()
        + input.strips.connectivity_len();
    let mut out_flat = Vec::with_capacity(total_out_pts * 3);

    let mut pd = PolyData::new();
    pd.verts = shrink_cells(&input.verts, pts, factor, &mut out_flat);
    pd.lines = shrink_cells(&input.lines, pts, factor, &mut out_flat);
    pd.polys = shrink_cells(&input.polys, pts, factor, &mut out_flat);
    pd.strips = shrink_cells(&input.strips, pts, factor, &mut out_flat);
    pd.points = Points::from_flat_vec(out_flat);
    *pd.cell_data_mut() = input.cell_data().clone();
    pd
}

fn shrink_cells(cells: &CellArray, pts: &[f64], factor: f64, out_flat: &mut Vec<f64>) -> CellArray {
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
        }
        out_off.push(out_conn.len() as i64);
    }

    CellArray::from_raw(out_off, out_conn)
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
    fn shrink_preserves_cell_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = shrink(&pd, 0.5);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.points.len(), 8);
    }
}
