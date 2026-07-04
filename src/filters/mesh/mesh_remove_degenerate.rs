//! Remove degenerate faces (zero area or collapsed edges).
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};
use crate::types::Scalar;

pub fn remove_degenerate_faces(mesh: &PolyData, min_area: f64) -> PolyData {
    let n = mesh.points.len();
    let mut polys = CellArray::new();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    for (poly_id, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell_points(cell, n) {
            continue;
        }
        let unique: std::collections::HashSet<i64> = cell.iter().copied().collect();
        if unique.len() < 3 {
            continue;
        }

        let area = polygon_area(mesh, cell);
        if area >= min_area {
            polys.push_cell(cell);
            old_poly_ids.push(poly_cell_offset + poly_id);
        }
    }
    let mut result = mesh.clone();
    result.polys = polys;
    remap_cell_data(mesh, &old_poly_ids, &mut result);
    result
}

fn polygon_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    let a = mesh.points.get(cell[0] as usize);
    let mut area = 0.0;
    for i in 1..cell.len() - 1 {
        let b = mesh.points.get(cell[i] as usize);
        let c = mesh.points.get(cell[i + 1] as usize);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        area += 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    }
    area
}

fn valid_cell_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| usize::try_from(id).ok().is_some_and(|idx| idx < num_points))
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
    restore_active_attributes(output.cell_data_mut(), input.cell_data());
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

fn restore_active_attributes(output: &mut DataSetAttributes, input: &DataSetAttributes) {
    if let Some(name) = input.scalars().map(|a| a.name()) {
        output.set_active_scalars(name);
    }
    if let Some(name) = input.vectors().map(|a| a.name()) {
        output.set_active_vectors(name);
    }
    if let Some(name) = input.normals().map(|a| a.name()) {
        output.set_active_normals(name);
    }
    if let Some(name) = input.tcoords().map(|a| a.name()) {
        output.set_active_tcoords(name);
    }
    if let Some(name) = input.tensors().map(|a| a.name()) {
        output.set_active_tensors(name);
    }
    if let Some(name) = input.global_ids().map(|a| a.name()) {
        output.set_active_global_ids(name);
    }
    if let Some(name) = input.pedigree_ids().map(|a| a.name()) {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = input.edge_flags().map(|a| a.name()) {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = input.tangents().map(|a| a.name()) {
        output.set_active_tangents(name);
    }
    if let Some(name) = input.rational_weights().map(|a| a.name()) {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = input.higher_order_degrees().map(|a| a.name()) {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = input.process_ids().map(|a| a.name()) {
        output.set_active_process_ids(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_degenerate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ], // last is degenerate
            vec![[0, 1, 2], [0, 1, 3]], // second face has zero area (0 and 3 are same point)
        );
        let r = remove_degenerate_faces(&mesh, 1e-10);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
