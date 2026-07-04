//! Extract faces whose vertex scalar values fall within a range.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn extract_scalar_range(
    mesh: &PolyData,
    scalar_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_tuples() >= n => a,
        None => return PolyData::new(),
        _ => return PolyData::new(),
    };
    let mut vals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals[i] = buf[0];
    }
    let mut used = vec![false; n];
    let mut kept = Vec::new();
    let mut selected_cell_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let use_global_cell_ids = mesh
        .cell_data()
        .iter()
        .any(|array| array.num_tuples() >= mesh.total_cells());
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.is_empty() {
            continue;
        }
        let Some(indices) = cell
            .iter()
            .map(|&v| valid_point_index(v, n))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        if !indices
            .iter()
            .all(|&vi| vals[vi] >= min_val && vals[vi] <= max_val)
        {
            continue;
        }
        for &vi in &indices {
            used[vi] = true;
        }
        kept.push(indices);
        selected_cell_ids.push(if use_global_cell_ids {
            poly_cell_offset + ci
        } else {
            ci
        });
    }
    let mut point_map = vec![0usize; n];
    let mut points = Points::<f64>::new();
    let mut selected_point_ids = Vec::new();
    for (i, is_used) in used.iter().copied().enumerate() {
        if is_used {
            point_map[i] = points.len();
            points.push(mesh.points.get(i));
            selected_point_ids.push(i);
        }
    }
    let mut polys = CellArray::new();
    for cell in kept {
        polys.push_cell(
            &cell
                .iter()
                .map(|&v| point_map[v] as i64)
                .collect::<Vec<_>>(),
        );
    }
    let mut result = PolyData::new();
    result.points = points;
    result.polys = polys;
    *result.field_data_mut() = mesh.field_data().clone();
    copy_attributes_by_indices(
        mesh.point_data(),
        result.point_data_mut(),
        &selected_point_ids,
    );
    copy_attributes_by_indices(mesh.cell_data(), result.cell_data_mut(), &selected_cell_ids);
    result
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn copy_attributes_by_indices(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    indices: &[usize],
) {
    for array in source.iter() {
        if indices.iter().all(|&idx| idx < array.num_tuples()) {
            target.add_array(copy_array_by_indices(array, indices));
        }
    }
    copy_active_attributes(source, target);
}

fn copy_array_by_indices(array: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(copy_typed_array($array, indices))
        };
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

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * num_components);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn copy_active_attributes(source: &DataSetAttributes, target: &mut DataSetAttributes) {
    if let Some(array) = source.scalars() {
        target.set_active_scalars(array.name());
    }
    if let Some(array) = source.vectors() {
        target.set_active_vectors(array.name());
    }
    if let Some(array) = source.normals() {
        target.set_active_normals(array.name());
    }
    if let Some(array) = source.tcoords() {
        target.set_active_tcoords(array.name());
    }
    if let Some(array) = source.tensors() {
        target.set_active_tensors(array.name());
    }
    if let Some(array) = source.global_ids() {
        target.set_active_global_ids(array.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_extract() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.5, 0.3, 1.0],
                1,
            )));
        let r = extract_scalar_range(&mesh, "v", 0.0, 0.6);
        assert_eq!(r.polys.num_cells(), 1); // only first face (all verts <= 0.6)
    }
}
