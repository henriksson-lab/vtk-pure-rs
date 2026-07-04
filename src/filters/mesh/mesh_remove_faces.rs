//! Remove faces by index or predicate.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Remove faces at given indices.
pub fn remove_faces_by_index(
    mesh: &PolyData,
    indices: &std::collections::HashSet<usize>,
) -> PolyData {
    remove_faces_if(mesh, |i, _| indices.contains(&i))
}

/// Remove faces matching a predicate.
pub fn remove_faces_if(mesh: &PolyData, pred: impl Fn(usize, &[i64]) -> bool) -> PolyData {
    let num_points = mesh.points.len();
    let mut kept_polys = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    for (poly_id, cell) in mesh.polys.iter().enumerate() {
        if !pred(poly_id, cell) && valid_cell_points(cell, num_points) {
            kept_polys.push((poly_cell_offset + poly_id, cell.to_vec()));
        }
    }

    let mut used = vec![false; num_points];
    mark_used(&mesh.verts, &mut used);
    mark_used(&mesh.lines, &mut used);
    mark_used(&mesh.strips, &mut used);
    for (_, cell) in &kept_polys {
        for &v in cell {
            used[v as usize] = true;
        }
    }

    let mut pt_map = vec![0usize; num_points];
    let mut pts = Points::<f64>::new();
    let mut original_point_ids = Vec::new();
    for i in 0..num_points {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
            original_point_ids.push(i);
        }
    }

    let mut old_cell_ids = Vec::new();
    let verts = remap_cells(&mesh.verts, &pt_map, &mut old_cell_ids, 0);
    let lines = remap_cells(
        &mesh.lines,
        &pt_map,
        &mut old_cell_ids,
        mesh.verts.num_cells(),
    );

    let mut polys = CellArray::new();
    for (old_cell_id, cell) in &kept_polys {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
        old_cell_ids.push(*old_cell_id);
    }

    let strip_offset = mesh.verts.num_cells() + mesh.lines.num_cells() + mesh.polys.num_cells();
    let strips = remap_cells(&mesh.strips, &pt_map, &mut old_cell_ids, strip_offset);

    let mut r = mesh.clone();
    r.points = pts;
    r.verts = verts;
    r.lines = lines;
    r.polys = polys;
    r.strips = strips;
    replace_point_data(
        r.point_data_mut(),
        mesh.point_data(),
        &original_point_ids,
        num_points,
    );
    remap_cell_data(mesh, &old_cell_ids, &mut r);
    r
}

/// Remove small faces (area below threshold).
pub fn remove_small_faces(mesh: &PolyData, min_area: f64) -> PolyData {
    remove_faces_if(mesh, |_, cell| {
        if cell.len() < 3 || !valid_cell_points(cell, mesh.points.len()) {
            return true;
        }
        polygon_area(mesh, cell) < min_area
    })
}

fn mark_used(cells: &CellArray, used: &mut [bool]) {
    for cell in cells.iter() {
        if !valid_cell_points(cell, used.len()) {
            continue;
        }
        for &v in cell {
            used[v as usize] = true;
        }
    }
}

fn remap_cells(
    cells: &CellArray,
    point_map: &[usize],
    old_cell_ids: &mut Vec<usize>,
    cell_offset: usize,
) -> CellArray {
    let mut output = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        if !valid_cell_points(cell, point_map.len()) {
            continue;
        }
        let mapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize] as i64).collect();
        output.push_cell(&mapped);
        old_cell_ids.push(cell_offset + cell_id);
    }
    output
}

fn polygon_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    let a = mesh.points.get(cell[0] as usize);
    let mut area = 0.0;
    for i in 1..cell.len() - 1 {
        let b = mesh.points.get(cell[i] as usize);
        let c = mesh.points.get(cell[i + 1] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
    }
    area
}

fn valid_cell_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| usize::try_from(id).ok().is_some_and(|idx| idx < num_points))
}

fn replace_point_data(
    output: &mut DataSetAttributes,
    input: &DataSetAttributes,
    original_ids: &[usize],
    num_input_points: usize,
) {
    let active_scalars = input.scalars().map(|a| a.name().to_string());
    let active_vectors = input.vectors().map(|a| a.name().to_string());
    let active_normals = input.normals().map(|a| a.name().to_string());
    let active_tcoords = input.tcoords().map(|a| a.name().to_string());
    let active_tensors = input.tensors().map(|a| a.name().to_string());
    let active_global_ids = input.global_ids().map(|a| a.name().to_string());
    let active_pedigree_ids = input.pedigree_ids().map(|a| a.name().to_string());
    let active_edge_flags = input.edge_flags().map(|a| a.name().to_string());
    let active_tangents = input.tangents().map(|a| a.name().to_string());
    let active_rational_weights = input.rational_weights().map(|a| a.name().to_string());
    let active_higher_order_degrees = input.higher_order_degrees().map(|a| a.name().to_string());
    let active_process_ids = input.process_ids().map(|a| a.name().to_string());

    output.clear();
    for array in input.field_data().iter() {
        if array.num_tuples() == num_input_points {
            output.add_array(compact_array(array, original_ids));
        }
    }

    if let Some(name) = active_scalars.as_deref() {
        output.set_active_scalars(name);
    }
    if let Some(name) = active_vectors.as_deref() {
        output.set_active_vectors(name);
    }
    if let Some(name) = active_normals.as_deref() {
        output.set_active_normals(name);
    }
    if let Some(name) = active_tcoords.as_deref() {
        output.set_active_tcoords(name);
    }
    if let Some(name) = active_tensors.as_deref() {
        output.set_active_tensors(name);
    }
    if let Some(name) = active_global_ids.as_deref() {
        output.set_active_global_ids(name);
    }
    if let Some(name) = active_pedigree_ids.as_deref() {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = active_edge_flags.as_deref() {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = active_tangents.as_deref() {
        output.set_active_tangents(name);
    }
    if let Some(name) = active_rational_weights.as_deref() {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = active_higher_order_degrees.as_deref() {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = active_process_ids.as_deref() {
        output.set_active_process_ids(name);
    }
}

fn compact_array(array: &AnyDataArray, original_ids: &[usize]) -> AnyDataArray {
    macro_rules! compact {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut data = Vec::with_capacity(original_ids.len() * num_components);
            for &source_id in original_ids {
                data.extend_from_slice($array.tuple(source_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), data, num_components))
        }};
    }

    match array {
        AnyDataArray::F32(a) => compact!(a, F32),
        AnyDataArray::F64(a) => compact!(a, F64),
        AnyDataArray::I8(a) => compact!(a, I8),
        AnyDataArray::I16(a) => compact!(a, I16),
        AnyDataArray::I32(a) => compact!(a, I32),
        AnyDataArray::I64(a) => compact!(a, I64),
        AnyDataArray::U8(a) => compact!(a, U8),
        AnyDataArray::U16(a) => compact!(a, U16),
        AnyDataArray::U32(a) => compact!(a, U32),
        AnyDataArray::U64(a) => compact!(a, U64),
    }
}

fn remap_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    if input.cell_data().num_arrays() == 0 {
        return;
    }

    output.cell_data_mut().clear();
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(remap_array(array, old_cell_ids));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_index() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        let mut s = std::collections::HashSet::new();
        s.insert(0);
        let r = remove_faces_by_index(&mesh, &s);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_remove_small() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, 10.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.01, 0.0, 0.0],
                [0.0, 0.01, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = remove_small_faces(&mesh, 0.01);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
