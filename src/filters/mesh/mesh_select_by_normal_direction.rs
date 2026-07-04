//! Select faces by normal direction (facing up, down, etc).
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
pub fn select_faces_facing(mesh: &PolyData, direction: [f64; 3], angle_threshold: f64) -> PolyData {
    let dl =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dl < 1e-15 {
        return PolyData::new();
    }
    let d = [direction[0] / dl, direction[1] / dl, direction[2] / dl];
    let cos_t = angle_threshold.to_radians().cos();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    let mut selected_cell_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let use_global_cell_ids = uses_global_cell_ids(mesh);
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if !cell
            .iter()
            .all(|&point_id| point_id >= 0 && (point_id as usize) < mesh.points.len())
        {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nl < 1e-15 {
            continue;
        }
        let dot = (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / nl;
        if dot >= cos_t {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
            selected_cell_ids.push(if use_global_cell_ids {
                poly_cell_offset + cell_id
            } else {
                cell_id
            });
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
pub fn select_upward_faces(mesh: &PolyData, angle: f64) -> PolyData {
    select_faces_facing(mesh, [0.0, 0.0, 1.0], angle)
}
pub fn select_downward_faces(mesh: &PolyData, angle: f64) -> PolyData {
    select_faces_facing(mesh, [0.0, 0.0, -1.0], angle)
}

fn uses_global_cell_ids(mesh: &PolyData) -> bool {
    mesh.cell_data()
        .iter()
        .any(|array| array.num_tuples() >= mesh.total_cells())
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_up() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, -1.0],
                [1.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = select_upward_faces(&m, 45.0);
        assert!(r.polys.num_cells() >= 1);
    }
}
