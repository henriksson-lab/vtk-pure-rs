//! Extract interior (non-boundary) vertices.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn extract_interior_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let ec = edge_counts(mesh);
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut selected_point_ids = Vec::new();
    for i in 0..n {
        if !boundary.contains(&i) {
            let idx = pts.len();
            pts.push(mesh.points.get(i));
            verts.push_cell(&[idx as i64]);
            selected_point_ids.push(i);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    *r.field_data_mut() = mesh.field_data().clone();
    copy_attributes_by_indices(mesh.point_data(), r.point_data_mut(), &selected_point_ids);
    r
}
pub fn count_interior_vertices(mesh: &PolyData) -> usize {
    let n = mesh.points.len();
    let ec = edge_counts(mesh);
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    n - boundary.len()
}

fn edge_counts(mesh: &PolyData) -> std::collections::HashMap<(usize, usize), usize> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        insert_polygon_edges(&mut ec, cell, mesh.points.len());
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut ec, tri[0], tri[1], mesh.points.len());
            insert_edge(&mut ec, tri[1], tri[2], mesh.points.len());
            insert_edge(&mut ec, tri[2], tri[0], mesh.points.len());
        }
    }
    ec
}

fn insert_polygon_edges(
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
    cell: &[i64],
    n_points: usize,
) {
    for i in 0..cell.len() {
        insert_edge(ec, cell[i], cell[(i + 1) % cell.len()], n_points);
    }
}

fn insert_edge(
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
    a: i64,
    b: i64,
    n_points: usize,
) {
    let (Some(a), Some(b)) = (
        valid_point_index(a, n_points),
        valid_point_index(b, n_points),
    ) else {
        return;
    };
    if a != b {
        *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
    }
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 3], [1, 4, 3], [0, 3, 2], [3, 4, 2]],
        );
        let c = count_interior_vertices(&m);
        assert!(c >= 1);
    }
}
