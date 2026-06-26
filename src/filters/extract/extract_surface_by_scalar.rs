//! Extract surface regions based on scalar value ranges.
//!
//! Combines threshold + extract_surface into a single operation.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Extract surface cells where a scalar is within [min_val, max_val].
///
/// More efficient than threshold + extract as it does both in one pass.
pub fn extract_surface_by_scalar(
    mesh: &PolyData,
    array_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == mesh.points.len() => a,
        _ => return PolyData::new(),
    };

    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut old_point_ids = Vec::new();
    let mut old_cell_ids = Vec::new();
    let mut buf = [0.0f64];
    let polys_offset = mesh.verts.num_cells() + mesh.lines.num_cells();

    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        // Check if all vertices are within range
        let all_in = cell.iter().all(|&pid| {
            arr.tuple_as_f64(pid as usize, &mut buf);
            buf[0] >= min_val && buf[0] <= max_val
        });
        if !all_in {
            continue;
        }

        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *pt_map.entry(old).or_insert_with(|| {
                let idx = new_points.len();
                new_points.push(mesh.points.get(old));
                old_point_ids.push(old);
                idx
            });
            new_ids.push(new_idx as i64);
        }
        new_polys.push_cell(&new_ids);
        old_cell_ids.push(polys_offset + cell_id);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    copy_point_data(mesh, &old_point_ids, &mut result);
    copy_cell_data(mesh, &old_cell_ids, &mut result);
    result
}

/// Extract surface cells where ANY vertex scalar exceeds threshold.
pub fn extract_surface_above_scalar(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == mesh.points.len() => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];

    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut old_point_ids = Vec::new();
    let mut old_cell_ids = Vec::new();
    let polys_offset = mesh.verts.num_cells() + mesh.lines.num_cells();

    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        let any_above = cell.iter().any(|&pid| {
            arr.tuple_as_f64(pid as usize, &mut buf);
            buf[0] >= threshold
        });
        if !any_above {
            continue;
        }

        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *pt_map.entry(old).or_insert_with(|| {
                let idx = new_points.len();
                new_points.push(mesh.points.get(old));
                old_point_ids.push(old);
                idx
            });
            new_ids.push(new_idx as i64);
        }
        new_polys.push_cell(&new_ids);
        old_cell_ids.push(polys_offset + cell_id);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    copy_point_data(mesh, &old_point_ids, &mut result);
    copy_cell_data(mesh, &old_cell_ids, &mut result);
    result
}

fn copy_point_data(input: &PolyData, old_point_ids: &[usize], output: &mut PolyData) {
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, old_point_ids));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn copy_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
                out.push_tuple($array.tuple(tuple_id));
            }
            AnyDataArray::$variant(out)
        }};
    }

    match array {
        AnyDataArray::F32(a) => select!(a, F32),
        AnyDataArray::F64(a) => select!(a, F64),
        AnyDataArray::I8(a) => select!(a, I8),
        AnyDataArray::I16(a) => select!(a, I16),
        AnyDataArray::I32(a) => select!(a, I32),
        AnyDataArray::I64(a) => select!(a, I64),
        AnyDataArray::U8(a) => select!(a, U8),
        AnyDataArray::U16(a) => select!(a, U16),
        AnyDataArray::U32(a) => select!(a, U32),
        AnyDataArray::U64(a) => select!(a, U64),
    }
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh() -> PolyData {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 10.0, 10.0, 50.0, 50.0, 50.0],
                1,
            )));
        m
    }

    #[test]
    fn by_range() {
        let result = extract_surface_by_scalar(&make_mesh(), "temp", 0.0, 20.0);
        assert_eq!(result.polys.num_cells(), 1); // only first triangle
    }

    #[test]
    fn above_threshold() {
        let result = extract_surface_above_scalar(&make_mesh(), "temp", 40.0);
        assert_eq!(result.polys.num_cells(), 1); // only second triangle
    }

    #[test]
    fn all_pass() {
        let result = extract_surface_by_scalar(&make_mesh(), "temp", 0.0, 100.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn none_pass() {
        let result = extract_surface_by_scalar(&make_mesh(), "temp", 90.0, 100.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
