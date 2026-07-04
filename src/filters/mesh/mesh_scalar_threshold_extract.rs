//! Extract sub-mesh where all vertices of a face satisfy scalar threshold.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
pub fn extract_where_all_above(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_where(mesh, array_name, |v| v >= threshold)
}
pub fn extract_where_all_below(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_where(mesh, array_name, |v| v <= threshold)
}
pub fn extract_where_any_above(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_where_any(mesh, array_name, |v| v >= threshold)
}
fn extract_where(mesh: &PolyData, array_name: &str, pred: impl Fn(f64) -> bool) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    if arr.num_tuples() != mesh.points.len() {
        return mesh.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    let mut kept_cell_ids = Vec::new();
    let cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let use_global_cell_ids = uses_global_cell_ids(mesh);
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        let Some(indices) = valid_cell_indices(cell, used.len()) else {
            continue;
        };
        if indices.iter().all(|&v| pred(vals[v])) {
            for &v in &indices {
                used[v] = true;
            }
            kept.push(indices);
            kept_cell_ids.push(if use_global_cell_ids {
                cell_offset + cell_id
            } else {
                cell_id
            });
        }
    }
    rebuild(mesh, &used, &kept, &kept_cell_ids)
}
fn extract_where_any(mesh: &PolyData, array_name: &str, pred: impl Fn(f64) -> bool) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    if arr.num_tuples() != mesh.points.len() {
        return mesh.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    let mut kept_cell_ids = Vec::new();
    let cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let use_global_cell_ids = uses_global_cell_ids(mesh);
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        let Some(indices) = valid_cell_indices(cell, used.len()) else {
            continue;
        };
        if indices.iter().any(|&v| pred(vals[v])) {
            for &v in &indices {
                used[v] = true;
            }
            kept.push(indices);
            kept_cell_ids.push(if use_global_cell_ids {
                cell_offset + cell_id
            } else {
                cell_id
            });
        }
    }
    rebuild(mesh, &used, &kept, &kept_cell_ids)
}
fn valid_cell_indices(cell: &[i64], num_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&point_id| usize::try_from(point_id).ok().filter(|&id| id < num_points))
        .collect()
}

fn uses_global_cell_ids(mesh: &PolyData) -> bool {
    mesh.cell_data()
        .iter()
        .any(|array| array.num_tuples() >= mesh.total_cells())
}

fn rebuild(
    mesh: &PolyData,
    used: &[bool],
    kept: &[Vec<usize>],
    kept_cell_ids: &[usize],
) -> PolyData {
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    let mut point_ids = Vec::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
            point_ids.push(i);
        }
    }
    let mut polys = CellArray::new();
    for c in kept {
        polys.push_cell(&c.iter().map(|&v| pm[v] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    *r.field_data_mut() = mesh.field_data().clone();
    copy_attribute_tuples(mesh.point_data(), r.point_data_mut(), &point_ids);
    copy_attribute_tuples(mesh.cell_data(), r.cell_data_mut(), kept_cell_ids);
    r
}
fn copy_attribute_tuples(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if tuple_ids.iter().all(|&id| id < array.num_tuples()) {
            target.add_array(subset_array(array, tuple_ids));
        }
    }
    copy_active_attributes(source, target);
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
    if let Some(array) = source.pedigree_ids() {
        target.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = source.edge_flags() {
        target.set_active_edge_flags(array.name());
    }
    if let Some(array) = source.tangents() {
        target.set_active_tangents(array.name());
    }
    if let Some(array) = source.rational_weights() {
        target.set_active_rational_weights(array.name());
    }
    if let Some(array) = source.higher_order_degrees() {
        target.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = source.process_ids() {
        target.set_active_process_ids(array.name());
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! subset {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut values = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                values.extend_from_slice($arr.tuple(tuple_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($arr.name(), values, nc))
        }};
    }
    match array {
        AnyDataArray::F32(arr) => subset!(arr, F32),
        AnyDataArray::F64(arr) => subset!(arr, F64),
        AnyDataArray::I8(arr) => subset!(arr, I8),
        AnyDataArray::I16(arr) => subset!(arr, I16),
        AnyDataArray::I32(arr) => subset!(arr, I32),
        AnyDataArray::I64(arr) => subset!(arr, I64),
        AnyDataArray::U8(arr) => subset!(arr, U8),
        AnyDataArray::U16(arr) => subset!(arr, U16),
        AnyDataArray::U32(arr) => subset!(arr, U32),
        AnyDataArray::U64(arr) => subset!(arr, U64),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_above() {
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
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 6.0, 3.0, 8.0, 9.0],
                1,
            )));
        let r = extract_where_all_above(&m, "s", 5.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_any() {
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
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 6.0, 3.0, 8.0, 9.0],
                1,
            )));
        let r = extract_where_any_above(&m, "s", 5.0);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
