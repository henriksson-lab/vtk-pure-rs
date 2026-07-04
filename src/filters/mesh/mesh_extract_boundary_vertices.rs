//! Extract boundary vertices as a point cloud.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn extract_boundary_vertices(mesh: &PolyData) -> PolyData {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let n = mesh.points.len();
    for cell in mesh.polys.iter() {
        count_polygon_edges(cell, n, &mut ec);
    }
    for strip in mesh.strips.iter() {
        count_triangle_strip_edges(strip, n, &mut ec);
    }
    let mut is_boundary = vec![false; n];
    for (&(a, b), &c) in &ec {
        if c == 1 {
            is_boundary[a] = true;
            is_boundary[b] = true;
        }
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut selected_point_ids = Vec::new();
    for (v, &boundary) in is_boundary.iter().enumerate() {
        if !boundary {
            continue;
        }
        let idx = pts.len();
        pts.push(mesh.points.get(v));
        verts.push_cell(&[idx as i64]);
        selected_point_ids.push(v);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    *r.field_data_mut() = mesh.field_data().clone();
    copy_attributes_by_indices(mesh.point_data(), r.point_data_mut(), &selected_point_ids);
    r
}
pub fn is_boundary_vertex(mesh: &PolyData, vertex: usize) -> bool {
    if vertex >= mesh.points.len() {
        return false;
    }
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let n = mesh.points.len();
    for cell in mesh.polys.iter() {
        count_polygon_edges(cell, n, &mut ec);
    }
    for strip in mesh.strips.iter() {
        count_triangle_strip_edges(strip, n, &mut ec);
    }
    ec.iter()
        .any(|(&(a, b), &c)| c == 1 && (a == vertex || b == vertex))
}

fn count_polygon_edges(
    cell: &[i64],
    n_points: usize,
    edge_counts: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    let nc = cell.len();
    if nc < 2 {
        return;
    }
    for i in 0..nc {
        let Some(a) = valid_point_index(cell[i], n_points) else {
            continue;
        };
        let Some(b) = valid_point_index(cell[(i + 1) % nc], n_points) else {
            continue;
        };
        if a == b {
            continue;
        }
        *edge_counts.entry((a.min(b), a.max(b))).or_insert(0) += 1;
    }
}

fn count_triangle_strip_edges(
    strip: &[i64],
    n_points: usize,
    edge_counts: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    for tri in strip.windows(3) {
        count_edge(tri[0], tri[1], n_points, edge_counts);
        count_edge(tri[1], tri[2], n_points, edge_counts);
        count_edge(tri[2], tri[0], n_points, edge_counts);
    }
}

fn count_edge(
    a: i64,
    b: i64,
    n_points: usize,
    edge_counts: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    let Some(a) = valid_point_index(a, n_points) else {
        return;
    };
    let Some(b) = valid_point_index(b, n_points) else {
        return;
    };
    if a == b {
        return;
    }
    *edge_counts.entry((a.min(b), a.max(b))).or_insert(0) += 1;
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
    fn test_extract() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extract_boundary_vertices(&m);
        assert_eq!(r.points.len(), 3);
    }
    #[test]
    fn test_is_boundary() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert!(is_boundary_vertex(&m, 0));
    }
}
