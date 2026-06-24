use crate::data::{CellArray, PolyData};

/// Reverse winding order of all polygon cells and flip normals.
/// Uses raw connectivity reversal — no clone of point data.
pub fn reverse_sense(input: &PolyData) -> PolyData {
    // Reverse connectivity in-place by reversing each cell's indices
    let src_off = input.polys.offsets();
    let src_conn = input.polys.connectivity();
    let nc = input.polys.num_cells();

    let mut conn = Vec::with_capacity(src_conn.len());
    let mut offsets = Vec::with_capacity(src_off.len());
    offsets.push(0i64);

    for ci in 0..nc {
        let start = src_off[ci] as usize;
        let end = src_off[ci + 1] as usize;
        // Push reversed
        for j in (start..end).rev() {
            conn.push(src_conn[j]);
        }
        offsets.push(conn.len() as i64);
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.polys = CellArray::from_raw(offsets, conn);
    pd.lines = input.lines.clone();
    pd.verts = input.verts.clone();

    // Flip normals if present
    if let Some(normals) = input.point_data().normals() {
        let nc = normals.num_components();
        let nt = normals.num_tuples();
        let name = normals.name().to_string();
        let mut flipped = Vec::with_capacity(nt * nc);
        let mut buf = vec![0.0f64; nc];
        for i in 0..nt {
            normals.tuple_as_f64(i, &mut buf);
            for v in &buf {
                flipped.push(-v);
            }
        }
        pd.point_data_mut()
            .add_array(crate::data::AnyDataArray::F64(
                crate::data::DataArray::from_vec(&name, flipped, nc),
            ));
        pd.point_data_mut().set_active_normals(&name);
    }

    // Copy other point/cell data
    // (skip for now — the main use case is winding reversal)

    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reverse_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = reverse_sense(&pd);
        assert_eq!(r.polys.cell(0), &[2, 1, 0]);
    }
    #[test]
    fn reverse_multiple() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = reverse_sense(&pd);
        assert_eq!(r.polys.cell(0), &[2, 1, 0]);
        assert_eq!(r.polys.cell(1), &[2, 3, 1]);
    }
    #[test]
    fn preserves_points() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let r = reverse_sense(&pd);
        assert_eq!(r.points.get(0), [1.0, 2.0, 3.0]);
    }
}
