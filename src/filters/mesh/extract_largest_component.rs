use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Extract the largest connected component from a PolyData mesh.
///
/// Uses union-find on shared vertices to identify connected components,
/// then keeps only cells belonging to the largest group.
pub fn extract_largest_component(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 || input.total_cells() == 0 {
        return PolyData::new();
    }

    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            let Some(indices) = cell_point_indices(cell, n) else {
                continue;
            };
            let first = indices[0];
            for &idx in &indices[1..] {
                union(&mut parent, &mut rank, first, idx);
            }
        }
    }

    let mut component_stats: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut global_cell_id = 0usize;
    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            let Some(indices) = cell_point_indices(cell, n) else {
                global_cell_id += 1;
                continue;
            };
            let root: usize = find(&mut parent, indices[0]);
            let stats = component_stats
                .entry(root)
                .or_insert((0usize, global_cell_id));
            stats.0 += 1;
            stats.1 = stats.1.min(global_cell_id);
            global_cell_id += 1;
        }
    }

    let largest_root: usize =
        match component_stats
            .iter()
            .max_by(|(_, &(count_a, first_a)), (_, &(count_b, first_b))| {
                count_a.cmp(&count_b).then_with(|| first_b.cmp(&first_a))
            }) {
            Some((&k, _)) => k,
            None => return PolyData::new(),
        };

    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut out_points: Points<f64> = Points::new();
    let mut selected_point_ids: Vec<usize> = Vec::new();
    let mut selected_cell_ids: Vec<usize> = Vec::new();

    let out_verts = collect_component_cells(
        &input.verts,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        0,
    );
    let line_offset = input.verts.num_cells();
    let out_lines = collect_component_cells(
        &input.lines,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        line_offset,
    );
    let poly_offset = line_offset + input.lines.num_cells();
    let out_polys = collect_component_cells(
        &input.polys,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        poly_offset,
    );
    let strip_offset = poly_offset + input.polys.num_cells();
    let out_strips = collect_component_cells(
        &input.strips,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
        &mut selected_point_ids,
        &mut selected_cell_ids,
        strip_offset,
    );

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.lines = out_lines;
    pd.polys = out_polys;
    pd.strips = out_strips;
    *pd.field_data_mut() = input.field_data().clone();
    copy_attributes_by_indices(input.point_data(), pd.point_data_mut(), &selected_point_ids);
    copy_attributes_by_indices(input.cell_data(), pd.cell_data_mut(), &selected_cell_ids);
    pd
}

fn valid_point_index(id: Option<i64>, n_points: usize) -> Option<usize> {
    usize::try_from(id?).ok().filter(|&id| id < n_points)
}

fn cell_point_indices(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    if cell.is_empty() {
        return None;
    }
    cell.iter()
        .map(|&id| valid_point_index(Some(id), n_points))
        .collect()
}

fn find(parent: &mut [usize], x: usize) -> usize {
    let mut r: usize = x;
    while parent[r] != r {
        parent[r] = parent[parent[r]];
        r = parent[r];
    }
    r
}

fn union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra: usize = find(parent, a);
    let rb: usize = find(parent, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        parent[ra] = rb;
    } else if rank[ra] > rank[rb] {
        parent[rb] = ra;
    } else {
        parent[rb] = ra;
        rank[ra] += 1;
    }
}

fn collect_component_cells(
    cells: &CellArray,
    input: &PolyData,
    parent: &mut [usize],
    largest_root: usize,
    point_map: &mut HashMap<usize, usize>,
    out_points: &mut Points<f64>,
    selected_point_ids: &mut Vec<usize>,
    selected_cell_ids: &mut Vec<usize>,
    cell_offset: usize,
) -> CellArray {
    let mut output = CellArray::new();
    for (local_cell_id, cell) in cells.iter().enumerate() {
        let Some(indices) = cell_point_indices(cell, input.points.len()) else {
            continue;
        };
        let root: usize = find(parent, indices[0]);
        if root != largest_root {
            continue;
        }
        let mut remapped = Vec::with_capacity(cell.len());
        for idx in indices {
            let next_id: usize = out_points.len();
            remapped.push(*point_map.entry(idx).or_insert_with(|| {
                out_points.push(input.points.get(idx));
                selected_point_ids.push(idx);
                next_id
            }) as i64);
        }
        output.push_cell(&remapped);
        selected_cell_ids.push(cell_offset + local_cell_id);
    }
    output
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
    fn single_component() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extract_largest_component(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn two_components_picks_larger() {
        // Component A: 2 triangles sharing edge 1-2
        // Component B: 1 triangle (disconnected vertices)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],    // 0 - comp A
                [1.0, 0.0, 0.0],    // 1 - comp A
                [0.5, 1.0, 0.0],    // 2 - comp A
                [1.5, 1.0, 0.0],    // 3 - comp A
                [10.0, 10.0, 10.0], // 4 - comp B
                [11.0, 10.0, 10.0], // 5 - comp B
                [10.5, 11.0, 10.0], // 6 - comp B
            ],
            vec![[0, 1, 2], [1, 2, 3], [4, 5, 6]],
        );
        let result = extract_largest_component(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = extract_largest_component(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn includes_non_polygon_cells() {
        let mut pd = PolyData::new();
        for i in 0..6 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = extract_largest_component(&pd);
        assert_eq!(result.lines.num_cells(), 2);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn preserves_selected_point_and_cell_data() {
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [10.0, 10.0, 10.0],
                [11.0, 10.0, 10.0],
                [10.5, 11.0, 10.0],
            ],
            vec![[0, 1, 2], [1, 2, 3], [4, 5, 6]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_ids",
                vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("point_ids");
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_ids",
                vec![10.0, 11.0, 12.0],
                1,
            )));

        let result = extract_largest_component(&pd);

        assert_eq!(
            result.point_data().scalars().unwrap().to_f64_vec(),
            vec![0.0, 1.0, 2.0, 3.0]
        );
        assert_eq!(
            result
                .cell_data()
                .get_array("cell_ids")
                .unwrap()
                .to_f64_vec(),
            vec![10.0, 11.0]
        );
    }

    #[test]
    fn invalid_cells_are_skipped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = extract_largest_component(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }
}
