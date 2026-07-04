//! Extract faces where cell scalar is within a range.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn extract_faces_by_cell_scalar(
    mesh: &PolyData,
    array_name: &str,
    lo: f64,
    hi: f64,
) -> PolyData {
    let arr = match mesh.cell_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= mesh.polys.num_cells() => a,
        _ => return PolyData::new(),
    };
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let use_global_cell_ids = arr.num_tuples() >= mesh.total_cells();
    let mut buf = [0.0f64];
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    let mut selected_cell_ids = Vec::new();
    for (ci, cell) in cells.iter().enumerate() {
        let cell_id = if use_global_cell_ids {
            poly_cell_offset + ci
        } else {
            ci
        };
        arr.tuple_as_f64(cell_id, &mut buf);
        if buf[0] >= lo
            && buf[0] <= hi
            && cell
                .iter()
                .all(|&v| valid_point_index(v, mesh.points.len()).is_some())
        {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.clone());
            selected_cell_ids.push(cell_id);
        }
    }
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    let mut selected_point_ids = Vec::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
            selected_point_ids.push(i);
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    *r.field_data_mut() = mesh.field_data().clone();
    copy_attributes_by_indices(mesh.point_data(), r.point_data_mut(), &selected_point_ids);
    copy_attributes_by_indices(mesh.cell_data(), r.cell_data_mut(), &selected_cell_ids);
    r
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
    if let Some(name) = source.scalars().map(|array| array.name().to_string()) {
        target.set_active_scalars(&name);
    }
    if let Some(name) = source.vectors().map(|array| array.name().to_string()) {
        target.set_active_vectors(&name);
    }
    if let Some(name) = source.normals().map(|array| array.name().to_string()) {
        target.set_active_normals(&name);
    }
    if let Some(name) = source.tcoords().map(|array| array.name().to_string()) {
        target.set_active_tcoords(&name);
    }
    if let Some(name) = source.tensors().map(|array| array.name().to_string()) {
        target.set_active_tensors(&name);
    }
    if let Some(name) = source.global_ids().map(|array| array.name().to_string()) {
        target.set_active_global_ids(&name);
    }
    if let Some(name) = source.pedigree_ids().map(|array| array.name().to_string()) {
        target.set_active_pedigree_ids(&name);
    }
    if let Some(name) = source.edge_flags().map(|array| array.name().to_string()) {
        target.set_active_edge_flags(&name);
    }
    if let Some(name) = source.tangents().map(|array| array.name().to_string()) {
        target.set_active_tangents(&name);
    }
    if let Some(name) = source
        .rational_weights()
        .map(|array| array.name().to_string())
    {
        target.set_active_rational_weights(&name);
    }
    if let Some(name) = source
        .higher_order_degrees()
        .map(|array| array.name().to_string())
    {
        target.set_active_higher_order_degrees(&name);
    }
    if let Some(name) = source.process_ids().map(|array| array.name().to_string()) {
        target.set_active_process_ids(&name);
    }
}
pub fn extract_faces_above_scalar(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_faces_by_cell_scalar(mesh, array_name, threshold, f64::INFINITY)
}
pub fn extract_faces_below_scalar(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_faces_by_cell_scalar(mesh, array_name, f64::NEG_INFINITY, threshold)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        m.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "q",
                vec![0.5, 0.9],
                1,
            )));
        let r = extract_faces_by_cell_scalar(&m, "q", 0.0, 0.6);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
