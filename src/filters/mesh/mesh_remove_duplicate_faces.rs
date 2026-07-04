//! Remove duplicate faces (faces with same vertex set regardless of order).
use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use crate::types::Scalar;

pub fn remove_duplicate_faces(mesh: &PolyData) -> PolyData {
    let mut seen = std::collections::HashSet::new();
    let mut polys = CellArray::new();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();

    for (poly_id, cell) in mesh.polys.iter().enumerate() {
        let mut key = cell.to_vec();
        key.sort();
        key.dedup();
        if key.len() == cell.len() && seen.insert(key) {
            polys.push_cell(cell);
            old_poly_ids.push(poly_cell_offset + poly_id);
        }
    }
    let mut result = mesh.clone();
    result.polys = polys;
    remap_cell_data(mesh, &old_poly_ids, &mut result);
    result
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
    fn test_dedup() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2], [0, 1, 2], [1, 0, 2]], // three copies (two duplicates)
        );
        let r = remove_duplicate_faces(&mesh);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
