use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Reverse winding order of all polygon cells and flip normals.
/// Mirrors `vtkReverseSense` with ReverseCellsOn and ReverseNormalsOn.
pub fn reverse_sense(input: &PolyData) -> PolyData {
    let mut pd = input.clone();

    pd.verts = reverse_cells(&input.verts);
    pd.lines = reverse_cells(&input.lines);
    pd.polys = reverse_cells(&input.polys);
    pd.strips = reverse_cells(&input.strips);

    // Flip normals if present.
    if let Some(normals) = input.point_data().normals() {
        let name = normals.name().to_string();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(flipped_normals(&name, normals)));
        pd.point_data_mut().set_active_normals(&name);
    }
    if let Some(normals) = input.cell_data().normals() {
        let name = normals.name().to_string();
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(flipped_normals(&name, normals)));
        pd.cell_data_mut().set_active_normals(&name);
    }

    pd
}

fn reverse_cells(cells: &CellArray) -> CellArray {
    let src_off = cells.offsets();
    let src_conn = cells.connectivity();
    let nc = cells.num_cells();
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

    CellArray::from_raw(offsets, conn)
}

fn flipped_normals(name: &str, normals: &AnyDataArray) -> DataArray<f64> {
    let nc = normals.num_components();
    let nt = normals.num_tuples();
    let mut flipped = Vec::with_capacity(nt * nc);
    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        normals.tuple_as_f64(i, &mut buf);
        for v in &buf {
            flipped.push(-v);
        }
    }
    DataArray::from_vec(name, flipped, nc)
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

    #[test]
    fn preserves_data_and_reverses_cell_families() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.strips.push_cell(&[0, 1, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "scalars",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let r = reverse_sense(&pd);

        assert_eq!(r.verts.cell(0), &[0]);
        assert_eq!(r.lines.cell(0), &[1, 0]);
        assert_eq!(r.polys.cell(0), &[2, 1, 0]);
        assert_eq!(r.strips.cell(0), &[3, 2, 1, 0]);
        assert!(r.point_data().get_array("scalars").is_some());
    }
}
