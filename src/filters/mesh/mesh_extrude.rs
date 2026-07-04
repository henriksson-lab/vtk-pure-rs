//! Mesh extrusion along normals or directions.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Extrude mesh along a direction vector, creating a solid shell.
pub fn extrude_direction(mesh: &PolyData, direction: [f64; 3], distance: f64) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let mut strips = CellArray::new();
    let mut line_cell_ids = Vec::new();
    let mut poly_cell_ids = Vec::new();
    let mut strip_cell_ids = Vec::new();

    // Original points
    for i in 0..n {
        pts.push(mesh.points.get(i));
    }
    // Offset points
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + direction[0] * distance,
            p[1] + direction[1] * distance,
            p[2] + direction[2] * distance,
        ]);
    }

    let offset = n as i64;

    for (cell_id, cell) in mesh.verts.iter().enumerate() {
        for &pt_id in cell {
            if valid_point_index(pt_id, n).is_some() {
                lines.push_cell(&[pt_id, pt_id + offset]);
                line_cell_ids.push(cell_id);
            }
        }
    }
    let line_offset = mesh.verts.num_cells();
    for (cell_id, cell) in mesh.lines.iter().enumerate() {
        for pair in cell.windows(2) {
            if valid_point_index(pair[0], n).is_some() && valid_point_index(pair[1], n).is_some() {
                strips.push_cell(&[pair[0], pair[1], pair[0] + offset, pair[1] + offset]);
                strip_cell_ids.push(line_offset + cell_id);
            }
        }
    }

    let poly_offset = line_offset + mesh.lines.num_cells();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if valid_polygon_cell(cell, n) {
            polys.push_cell(cell);
            poly_cell_ids.push(poly_offset + cell_id);
        }
    }
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if valid_polygon_cell(cell, n) {
            let top: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            polys.push_cell(&top);
            poly_cell_ids.push(poly_offset + cell_id);
        }
    }
    let strip_offset = poly_offset + mesh.polys.num_cells();
    for (cell_id, cell) in mesh.strips.iter().enumerate() {
        if valid_strip_cell(cell, n) {
            strips.push_cell(cell);
            strip_cell_ids.push(strip_offset + cell_id);
            let top: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            strips.push_cell(&top);
            strip_cell_ids.push(strip_offset + cell_id);
        }
    }

    for (a, b, cell_id) in boundary_edges(mesh) {
        strips.push_cell(&[a, b, a + offset, b + offset]);
        strip_cell_ids.push(cell_id);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    *result.field_data_mut() = mesh.field_data().clone();
    copy_duplicated_point_data(mesh.point_data(), result.point_data_mut(), n);
    let mut cell_ids =
        Vec::with_capacity(line_cell_ids.len() + poly_cell_ids.len() + strip_cell_ids.len());
    cell_ids.extend(line_cell_ids);
    cell_ids.extend(poly_cell_ids);
    cell_ids.extend(strip_cell_ids);
    copy_cell_data_by_ids(mesh.cell_data(), result.cell_data_mut(), &cell_ids);
    result
}

/// Extrude edges of a mesh radially from centroid.
pub fn extrude_radial(mesh: &PolyData, distance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let mut strips = CellArray::new();
    let mut line_cell_ids = Vec::new();
    let mut poly_cell_ids = Vec::new();
    let mut strip_cell_ids = Vec::new();
    for i in 0..n {
        pts.push(mesh.points.get(i));
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-15);
        pts.push([
            p[0] + dx / len * distance,
            p[1] + dy / len * distance,
            p[2] + dz / len * distance,
        ]);
    }

    let offset = n as i64;

    for (cell_id, cell) in mesh.verts.iter().enumerate() {
        for &pt_id in cell {
            if valid_point_index(pt_id, n).is_some() {
                lines.push_cell(&[pt_id, pt_id + offset]);
                line_cell_ids.push(cell_id);
            }
        }
    }
    let line_offset = mesh.verts.num_cells();
    for (cell_id, cell) in mesh.lines.iter().enumerate() {
        for pair in cell.windows(2) {
            if valid_point_index(pair[0], n).is_some() && valid_point_index(pair[1], n).is_some() {
                strips.push_cell(&[pair[0], pair[1], pair[0] + offset, pair[1] + offset]);
                strip_cell_ids.push(line_offset + cell_id);
            }
        }
    }
    let poly_offset = line_offset + mesh.lines.num_cells();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if valid_polygon_cell(cell, n) {
            polys.push_cell(cell);
            poly_cell_ids.push(poly_offset + cell_id);
        }
    }
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if valid_polygon_cell(cell, n) {
            let top: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            polys.push_cell(&top);
            poly_cell_ids.push(poly_offset + cell_id);
        }
    }
    let strip_offset = poly_offset + mesh.polys.num_cells();
    for (cell_id, cell) in mesh.strips.iter().enumerate() {
        if valid_strip_cell(cell, n) {
            strips.push_cell(cell);
            strip_cell_ids.push(strip_offset + cell_id);
            let top: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            strips.push_cell(&top);
            strip_cell_ids.push(strip_offset + cell_id);
        }
    }

    for (a, b, cell_id) in boundary_edges(mesh) {
        strips.push_cell(&[a, b, a + offset, b + offset]);
        strip_cell_ids.push(cell_id);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    *result.field_data_mut() = mesh.field_data().clone();
    copy_duplicated_point_data(mesh.point_data(), result.point_data_mut(), n);
    let mut cell_ids =
        Vec::with_capacity(line_cell_ids.len() + poly_cell_ids.len() + strip_cell_ids.len());
    cell_ids.extend(line_cell_ids);
    cell_ids.extend(poly_cell_ids);
    cell_ids.extend(strip_cell_ids);
    copy_cell_data_by_ids(mesh.cell_data(), result.cell_data_mut(), &cell_ids);
    result
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn valid_cell(cell: &[i64], n_points: usize) -> bool {
    cell.iter()
        .all(|&id| valid_point_index(id, n_points).is_some())
}

fn valid_polygon_cell(cell: &[i64], n_points: usize) -> bool {
    cell.len() >= 3 && valid_cell(cell, n_points)
}

fn valid_strip_cell(cell: &[i64], n_points: usize) -> bool {
    cell.windows(3).any(|tri| valid_triangle(tri, n_points))
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && valid_cell(tri, n_points)
}

fn boundary_edges(mesh: &PolyData) -> Vec<(i64, i64, usize)> {
    let n = mesh.points.len();
    let mut edge_counts: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut ordered_edges = Vec::new();
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if !valid_polygon_cell(cell, n) {
            continue;
        }
        count_polygon_edges(
            cell,
            n,
            poly_offset + cell_id,
            &mut edge_counts,
            &mut ordered_edges,
        );
    }
    let strip_offset = poly_offset + mesh.polys.num_cells();
    for (cell_id, strip) in mesh.strips.iter().enumerate() {
        for (i, tri) in strip.windows(3).enumerate() {
            if !valid_triangle(tri, n) {
                continue;
            }
            let tri = if i % 2 == 0 {
                [tri[0], tri[1], tri[2]]
            } else {
                [tri[1], tri[0], tri[2]]
            };
            count_polygon_edges(
                &tri,
                n,
                strip_offset + cell_id,
                &mut edge_counts,
                &mut ordered_edges,
            );
        }
    }
    ordered_edges
        .into_iter()
        .filter_map(|(a, b, cell_id, key)| (edge_counts[&key] == 1).then_some((a, b, cell_id)))
        .collect()
}

fn count_polygon_edges(
    cell: &[i64],
    n_points: usize,
    cell_id: usize,
    edge_counts: &mut std::collections::HashMap<(usize, usize), usize>,
    ordered_edges: &mut Vec<(i64, i64, usize, (usize, usize))>,
) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        let Some(a) = valid_point_index(cell[i], n_points) else {
            continue;
        };
        let Some(b) = valid_point_index(cell[(i + 1) % cell.len()], n_points) else {
            continue;
        };
        if a == b {
            continue;
        }
        let key = (a.min(b), a.max(b));
        *edge_counts.entry(key).or_insert(0) += 1;
        ordered_edges.push((cell[i], cell[(i + 1) % cell.len()], cell_id, key));
    }
}

fn copy_duplicated_point_data(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    num_points: usize,
) {
    let normals_name = source.normals().map(|array| array.name());
    for array in source.iter() {
        if array.num_tuples() >= num_points && Some(array.name()) != normals_name {
            target.add_array(copy_duplicated_array(array, num_points));
        }
    }
    copy_active_attributes(source, target, false);
}

fn copy_cell_data_by_ids(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    ids: &[usize],
) {
    if ids.is_empty() {
        return;
    }
    let normals_name = source.normals().map(|array| array.name());
    for array in source.iter() {
        if ids.iter().all(|&id| id < array.num_tuples()) && Some(array.name()) != normals_name {
            target.add_array(copy_array_by_indices(array, ids));
        }
    }
    copy_active_attributes(source, target, false);
}

fn copy_duplicated_array(array: &AnyDataArray, num_points: usize) -> AnyDataArray {
    let mut ids = Vec::with_capacity(num_points * 2);
    ids.extend(0..num_points);
    ids.extend(0..num_points);
    copy_array_by_indices(array, &ids)
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

fn copy_active_attributes(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    copy_normals: bool,
) {
    if let Some(name) = source.scalars().map(|array| array.name().to_string()) {
        target.set_active_scalars(&name);
    }
    if let Some(name) = source.vectors().map(|array| array.name().to_string()) {
        target.set_active_vectors(&name);
    }
    if copy_normals {
        if let Some(name) = source.normals().map(|array| array.name().to_string()) {
            target.set_active_normals(&name);
        }
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
    fn test_extrude_dir() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extrude_direction(&mesh, [0.0, 0.0, 1.0], 2.0);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 2);
        assert_eq!(r.strips.num_cells(), 3);
    }
    #[test]
    fn test_extrude_radial() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extrude_radial(&mesh, 0.5);
        assert_eq!(r.points.len(), 6);
    }
}
