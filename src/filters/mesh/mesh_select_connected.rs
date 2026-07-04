//! Select connected region containing a seed vertex.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashMap;

/// Select the connected component containing the given seed vertex.
pub fn select_connected_region(mesh: &PolyData, seed: usize) -> PolyData {
    let n = mesh.points.len();
    if seed >= n {
        return PolyData::new();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        for cell in cells.iter() {
            let Some(indices) = cell_point_indices(cell, n) else {
                continue;
            };
            if indices.len() < 2 {
                continue;
            }
            for i in 0..indices.len() {
                let a = indices[i];
                let b = indices[(i + 1) % indices.len()];
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(seed);
    visited[seed] = true;
    while let Some(v) = queue.pop_front() {
        for &u in &nb[v] {
            if !visited[u] {
                visited[u] = true;
                queue.push_back(u);
            }
        }
    }
    let mut point_map = HashMap::new();
    let mut points = Points::<f64>::new();
    let mut selected_point_ids = Vec::new();
    let mut selected_cell_ids = Vec::new();
    let verts = collect_connected_cells(
        &mesh.verts,
        mesh,
        &visited,
        &mut point_map,
        &mut points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        0,
    );
    let line_offset = mesh.verts.num_cells();
    let lines = collect_connected_cells(
        &mesh.lines,
        mesh,
        &visited,
        &mut point_map,
        &mut points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        line_offset,
    );
    let poly_offset = line_offset + mesh.lines.num_cells();
    let polys = collect_connected_cells(
        &mesh.polys,
        mesh,
        &visited,
        &mut point_map,
        &mut points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        poly_offset,
    );
    let strip_offset = poly_offset + mesh.polys.num_cells();
    let strips = collect_connected_cells(
        &mesh.strips,
        mesh,
        &visited,
        &mut point_map,
        &mut points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        strip_offset,
    );

    let mut r = PolyData::new();
    r.points = points;
    r.verts = verts;
    r.lines = lines;
    r.polys = polys;
    r.strips = strips;
    *r.field_data_mut() = mesh.field_data().clone();
    copy_attributes_by_indices(mesh.point_data(), r.point_data_mut(), &selected_point_ids);
    copy_attributes_by_indices(mesh.cell_data(), r.cell_data_mut(), &selected_cell_ids);
    r
}

fn collect_connected_cells(
    cells: &CellArray,
    mesh: &PolyData,
    visited: &[bool],
    point_map: &mut HashMap<usize, usize>,
    points: &mut Points<f64>,
    selected_point_ids: &mut Vec<usize>,
    selected_cell_ids: &mut Vec<usize>,
    cell_offset: usize,
) -> CellArray {
    let mut output = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let Some(indices) = cell_point_indices(cell, mesh.points.len()) else {
            continue;
        };
        if !indices.iter().all(|&v| visited[v]) {
            continue;
        }
        let mut remapped = Vec::with_capacity(indices.len());
        for v in indices {
            let next_id = points.len();
            let mapped_id = point_map.entry(v).or_insert_with(|| {
                points.push(mesh.points.get(v));
                selected_point_ids.push(v);
                next_id
            });
            remapped.push(*mapped_id as i64);
        }
        output.push_cell(&remapped);
        selected_cell_ids.push(cell_offset + cell_id);
    }
    output
}

fn cell_point_indices(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
        .collect()
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

/// Count connected components.
pub fn count_connected_components(mesh: &PolyData) -> usize {
    let n = mesh.points.len();
    let mut parent: Vec<usize> = (0..n).collect();
    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        for cell in cells.iter() {
            let Some(indices) = cell_point_indices(cell, n) else {
                continue;
            };
            let Some((&first, rest)) = indices.split_first() else {
                continue;
            };
            for &idx in rest {
                union(&mut parent, first, idx);
            }
        }
    }
    let mut roots = std::collections::HashSet::new();
    let used: std::collections::HashSet<usize> =
        [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips]
            .into_iter()
            .flat_map(|cells| cells.iter())
            .filter_map(|cell| cell_point_indices(cell, n))
            .flatten()
            .collect();
    for &v in &used {
        roots.insert(find(&mut parent, v));
    }
    roots.len()
}

fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra != rb {
        p[rb] = ra;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_select() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = select_connected_region(&mesh, 0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_count() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        assert_eq!(count_connected_components(&mesh), 2);
    }

    #[test]
    fn test_shared_zero_point_not_duplicated() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let r = select_connected_region(&mesh, 0);
        assert_eq!(r.points.len(), 4);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
