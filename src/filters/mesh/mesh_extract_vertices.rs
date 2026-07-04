//! Extract vertices whose scalar value is in a given range as a point cloud.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn extract_vertices_by_scalar(
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
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut buf = [0.0f64];
    let mut selected_point_ids = Vec::new();
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= min_val && buf[0] <= max_val {
            let idx = pts.len();
            pts.push(mesh.points.get(i));
            verts.push_cell(&[idx as i64]);
            selected_point_ids.push(i);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.verts = verts;
    *m.field_data_mut() = mesh.field_data().clone();
    copy_attributes_by_indices(mesh.point_data(), m.point_data_mut(), &selected_point_ids);
    m
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
    fn test_extract_verts() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 5.0, 10.0],
                1,
            )));
        let r = extract_vertices_by_scalar(&mesh, "v", 3.0, 7.0);
        assert_eq!(r.points.len(), 1); // only vertex 1 (value 5.0)
    }
}
