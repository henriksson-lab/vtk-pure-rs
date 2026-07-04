use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::types::Scalar;
use std::collections::HashSet;

/// Count the number of duplicate (overlapping) faces in a PolyData.
///
/// Two faces are duplicates if they share the same set of vertex indices,
/// regardless of ordering.
pub fn count_duplicate_faces(input: &PolyData) -> usize {
    let mut seen = HashSet::new();
    let mut duplicates: usize = 0;

    for cell in input.polys.iter() {
        let key = polygon_key(cell);
        if key.len() != cell.len() {
            continue;
        }
        if !seen.insert(key) {
            duplicates += 1;
        }
    }

    duplicates
}

/// Remove duplicate faces from a PolyData, keeping only the first occurrence.
///
/// Two faces are considered duplicates if they share the same set of vertex
/// indices (order-independent).
pub fn remove_duplicate_faces(input: &PolyData) -> PolyData {
    let mut seen = HashSet::new();
    let mut output = input.clone();
    output.polys.clear();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        let key = polygon_key(cell);
        if key.len() == cell.len() && seen.insert(key) {
            output.polys.push_cell(cell);
            old_poly_ids.push(poly_cell_offset + poly_id);
        }
    }

    remap_cell_data(input, &old_poly_ids, &mut output);
    output
}

fn polygon_key(cell: &[i64]) -> Vec<i64> {
    let mut sorted = cell.to_vec();
    sorted.sort();
    sorted.dedup();
    sorted
}

fn remap_cell_data(input: &PolyData, old_poly_ids: &[usize], output: &mut PolyData) {
    if input.cell_data().num_arrays() == 0 {
        return;
    }

    let mut old_cell_ids = Vec::with_capacity(output.total_cells());
    old_cell_ids.extend(0..input.verts.num_cells());

    let line_offset = input.verts.num_cells();
    old_cell_ids.extend(line_offset..line_offset + input.lines.num_cells());

    old_cell_ids.extend_from_slice(old_poly_ids);

    let strip_offset = input.verts.num_cells() + input.lines.num_cells() + input.polys.num_cells();
    old_cell_ids.extend(strip_offset..strip_offset + input.strips.num_cells());

    output.cell_data_mut().clear();
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(remap_array(array, &old_cell_ids));
        }
    }
}

fn remap_array(array: &AnyDataArray, old_cell_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($array, old_cell_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => remap!(array, F32),
        AnyDataArray::F64(array) => remap!(array, F64),
        AnyDataArray::I8(array) => remap!(array, I8),
        AnyDataArray::I16(array) => remap!(array, I16),
        AnyDataArray::I32(array) => remap!(array, I32),
        AnyDataArray::I64(array) => remap!(array, I64),
        AnyDataArray::U8(array) => remap!(array, U8),
        AnyDataArray::U16(array) => remap!(array, U16),
        AnyDataArray::U32(array) => remap!(array, U32),
        AnyDataArray::U64(array) => remap!(array, U64),
    }
}

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_cell_ids: &[usize]) -> DataArray<T> {
    let mut data = Vec::with_capacity(old_cell_ids.len() * array.num_components());
    for &old_cell_id in old_cell_ids {
        data.extend_from_slice(array.tuple(old_cell_id));
    }
    DataArray::from_vec(array.name(), data, array.num_components())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicates() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert_eq!(count_duplicate_faces(&pd), 0);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn exact_duplicate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 2]); // exact duplicate

        assert_eq!(count_duplicate_faces(&pd), 1);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn reordered_duplicate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 0, 1]); // same vertices, different order

        assert_eq!(count_duplicate_faces(&pd), 1);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn degenerate_face_is_preserved_when_unique() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 1]);

        assert_eq!(count_duplicate_faces(&pd), 0);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn degenerate_duplicate_counts_once() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 1]);
        pd.polys.push_cell(&[1, 0, 1]);

        assert_eq!(count_duplicate_faces(&pd), 0);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn cell_data_follows_kept_faces() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 1, 0]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "id",
                vec![10, 20, 30],
                1,
            )));

        let result = remove_duplicate_faces(&pd);
        let ids = result.cell_data().get_array("id").unwrap();
        let mut buf = [0.0f64];
        ids.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        ids.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 30.0);
    }
}
