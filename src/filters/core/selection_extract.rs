use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData, Selection,
    SelectionContentType, SelectionFieldType,
};

/// Apply a Selection to a PolyData to extract the selected subset.
///
/// For point-based index selections: extracts selected points and cells
/// that use only selected points.
/// For cell-based index selections: extracts selected cells and their points.
/// For threshold selections: selects points/cells by scalar value range.
pub fn extract_selection(input: &PolyData, selection: &Selection) -> PolyData {
    if selection.num_nodes() == 0 {
        return input.clone();
    }

    // Combine all nodes to get selected indices
    let node = &selection.nodes()[0]; // use first node
    match node.content_type {
        SelectionContentType::Indices => {
            let indices: Vec<usize> = node
                .selection_list
                .iter()
                .filter_map(|&v| (v >= 0.0).then_some(v as usize))
                .collect();
            match node.field_type {
                SelectionFieldType::Point => extract_by_point_indices(input, &indices),
                SelectionFieldType::Cell => extract_by_cell_indices(input, &indices),
                SelectionFieldType::Field
                | SelectionFieldType::Vertex
                | SelectionFieldType::Edge
                | SelectionFieldType::Row => PolyData::new(),
            }
        }
        SelectionContentType::Thresholds => {
            if node.selection_list.len() < 2 {
                return PolyData::new();
            }
            let min = node.selection_list[0];
            let max = node.selection_list[1];
            let array_name = node.array_name.as_deref().unwrap_or("");
            match node.field_type {
                SelectionFieldType::Point => {
                    extract_by_point_threshold(input, array_name, min, max)
                }
                SelectionFieldType::Cell => extract_by_cell_threshold(input, array_name, min, max),
                SelectionFieldType::Field
                | SelectionFieldType::Vertex
                | SelectionFieldType::Edge
                | SelectionFieldType::Row => PolyData::new(),
            }
        }
        _ => input.clone(),
    }
}

fn extract_by_point_indices(input: &PolyData, indices: &[usize]) -> PolyData {
    let mut pd = PolyData::new();
    let n = input.points.len();

    let mut old_to_new = vec![usize::MAX; n];
    let mut point_ids = Vec::new();
    for &old_idx in indices {
        if old_idx < n && old_to_new[old_idx] == usize::MAX {
            let new_idx = pd.points.len();
            pd.points.push(input.points.get(old_idx));
            old_to_new[old_idx] = new_idx;
            point_ids.push(old_idx);
        }
    }

    let mut cell_ids = Vec::new();
    copy_cells_using_selected_points(&input.verts, &mut pd.verts, &old_to_new, 0, &mut cell_ids);
    let lines_offset = input.verts.num_cells();
    copy_cells_using_selected_points(
        &input.lines,
        &mut pd.lines,
        &old_to_new,
        lines_offset,
        &mut cell_ids,
    );
    let polys_offset = lines_offset + input.lines.num_cells();
    copy_cells_using_selected_points(
        &input.polys,
        &mut pd.polys,
        &old_to_new,
        polys_offset,
        &mut cell_ids,
    );
    let strips_offset = polys_offset + input.polys.num_cells();
    copy_cells_using_selected_points(
        &input.strips,
        &mut pd.strips,
        &old_to_new,
        strips_offset,
        &mut cell_ids,
    );

    copy_tuple_subset(
        input.point_data(),
        pd.point_data_mut(),
        &point_ids,
        input.points.len(),
    );
    copy_tuple_subset(
        input.cell_data(),
        pd.cell_data_mut(),
        &cell_ids,
        input.total_cells(),
    );
    pd
}

fn extract_by_cell_indices(input: &PolyData, indices: &[usize]) -> PolyData {
    let mut pd = PolyData::new();
    let n = input.points.len();
    let mut point_used = vec![false; n];
    let mut valid_cell_ids = Vec::new();

    for &ci in indices {
        if let Some(cell) = cell_by_global_id(input, ci) {
            valid_cell_ids.push(ci);
            for &pid in cell {
                if pid >= 0 {
                    let pid = pid as usize;
                    if pid < n {
                        point_used[pid] = true;
                    }
                }
            }
        }
    }

    let mut old_to_new = vec![usize::MAX; n];
    let mut point_ids = Vec::new();
    for (i, &used) in point_used.iter().enumerate() {
        if used {
            old_to_new[i] = pd.points.len();
            pd.points.push(input.points.get(i));
            point_ids.push(i);
        }
    }

    for &ci in &valid_cell_ids {
        let Some((kind, cell)) = cell_kind_by_global_id(input, ci) else {
            continue;
        };
        match kind {
            CellKind::Verts => push_remapped_cell(&mut pd.verts, cell, &old_to_new),
            CellKind::Lines => push_remapped_cell(&mut pd.lines, cell, &old_to_new),
            CellKind::Polys => push_remapped_cell(&mut pd.polys, cell, &old_to_new),
            CellKind::Strips => push_remapped_cell(&mut pd.strips, cell, &old_to_new),
        }
    }

    copy_tuple_subset(
        input.point_data(),
        pd.point_data_mut(),
        &point_ids,
        input.points.len(),
    );
    copy_tuple_subset(
        input.cell_data(),
        pd.cell_data_mut(),
        &valid_cell_ids,
        input.total_cells(),
    );
    pd
}

#[derive(Clone, Copy)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

fn cell_by_global_id(input: &PolyData, cell_id: usize) -> Option<&[i64]> {
    cell_kind_by_global_id(input, cell_id).map(|(_, cell)| cell)
}

fn cell_kind_by_global_id(input: &PolyData, cell_id: usize) -> Option<(CellKind, &[i64])> {
    let mut offset = 0;
    if cell_id < offset + input.verts.num_cells() {
        return Some((CellKind::Verts, input.verts.cell(cell_id - offset)));
    }
    offset += input.verts.num_cells();
    if cell_id < offset + input.lines.num_cells() {
        return Some((CellKind::Lines, input.lines.cell(cell_id - offset)));
    }
    offset += input.lines.num_cells();
    if cell_id < offset + input.polys.num_cells() {
        return Some((CellKind::Polys, input.polys.cell(cell_id - offset)));
    }
    offset += input.polys.num_cells();
    if cell_id < offset + input.strips.num_cells() {
        return Some((CellKind::Strips, input.strips.cell(cell_id - offset)));
    }
    None
}

fn copy_cells_using_selected_points(
    input: &CellArray,
    output: &mut CellArray,
    old_to_new: &[usize],
    cell_offset: usize,
    cell_ids: &mut Vec<usize>,
) {
    for (local_id, cell) in input.iter().enumerate() {
        let mut new_cell = Vec::with_capacity(cell.len());
        let mut selected = true;
        for &pid in cell {
            if pid < 0 {
                selected = false;
                break;
            }
            let pid = pid as usize;
            if pid >= old_to_new.len() || old_to_new[pid] == usize::MAX {
                selected = false;
                break;
            }
            new_cell.push(old_to_new[pid] as i64);
        }
        if selected {
            output.push_cell(&new_cell);
            cell_ids.push(cell_offset + local_id);
        }
    }
}

fn push_remapped_cell(output: &mut CellArray, cell: &[i64], old_to_new: &[usize]) {
    let mut new_cell = Vec::with_capacity(cell.len());
    for &pid in cell {
        if pid < 0 {
            return;
        }
        let pid = pid as usize;
        if pid >= old_to_new.len() || old_to_new[pid] == usize::MAX {
            return;
        }
        new_cell.push(old_to_new[pid] as i64);
    }
    output.push_cell(&new_cell);
}

fn extract_by_point_threshold(input: &PolyData, array_name: &str, min: f64, max: f64) -> PolyData {
    let Some(arr) = input.point_data().get_array(array_name) else {
        return PolyData::new();
    };

    let mut selected = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= min && buf[0] <= max {
            selected.push(i);
        }
    }

    extract_by_point_indices(input, &selected)
}

fn extract_by_cell_threshold(input: &PolyData, array_name: &str, min: f64, max: f64) -> PolyData {
    let Some(arr) = input.cell_data().get_array(array_name) else {
        return PolyData::new();
    };

    let mut selected = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= min && buf[0] <= max {
            selected.push(i);
        }
    }

    extract_by_cell_indices(input, &selected)
}

fn copy_tuple_subset(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
    expected_tuples: usize,
) {
    for array in source.iter() {
        if array.num_tuples() != expected_tuples {
            continue;
        }
        if let Some(subset) = subset_array(array, tuple_ids) {
            let name = subset.name().to_string();
            target.add_array(subset);
            copy_active_attribute(source, target, &name);
        }
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            let nc = a.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                if tuple_id >= a.num_tuples() {
                    return None;
                }
                data.extend_from_slice(a.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                a.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(_) => subset_variant!(F32),
        AnyDataArray::F64(_) => subset_variant!(F64),
        AnyDataArray::I8(_) => subset_variant!(I8),
        AnyDataArray::I16(_) => subset_variant!(I16),
        AnyDataArray::I32(_) => subset_variant!(I32),
        AnyDataArray::I64(_) => subset_variant!(I64),
        AnyDataArray::U8(_) => subset_variant!(U8),
        AnyDataArray::U16(_) => subset_variant!(U16),
        AnyDataArray::U32(_) => subset_variant!(U32),
        AnyDataArray::U64(_) => subset_variant!(U64),
    }
}

fn copy_active_attribute(source: &DataSetAttributes, target: &mut DataSetAttributes, name: &str) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.normals().map(|a| a.name()) == Some(name) {
        target.set_active_normals(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
    if source.tensors().map(|a| a.name()) == Some(name) {
        target.set_active_tensors(name);
    }
    if source.global_ids().map(|a| a.name()) == Some(name) {
        target.set_active_global_ids(name);
    }
    if source.pedigree_ids().map(|a| a.name()) == Some(name) {
        target.set_active_pedigree_ids(name);
    }
    if source.edge_flags().map(|a| a.name()) == Some(name) {
        target.set_active_edge_flags(name);
    }
    if source.tangents().map(|a| a.name()) == Some(name) {
        target.set_active_tangents(name);
    }
    if source.rational_weights().map(|a| a.name()) == Some(name) {
        target.set_active_rational_weights(name);
    }
    if source.higher_order_degrees().map(|a| a.name()) == Some(name) {
        target.set_active_higher_order_degrees(name);
    }
    if source.process_ids().map(|a| a.name()) == Some(name) {
        target.set_active_process_ids(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, SelectionNode};

    #[test]
    fn extract_by_points() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let mut sel = Selection::new();
        sel.add_node(SelectionNode::from_point_indices(vec![0, 1, 2]));

        let result = extract_selection(&pd, &sel);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1); // only first tri uses all 3
    }

    #[test]
    fn extract_by_cells() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let mut sel = Selection::new();
        sel.add_node(SelectionNode::from_cell_indices(vec![1]));

        let result = extract_selection(&pd, &sel);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3); // 3 unique points in cell 1
    }

    #[test]
    fn extract_by_threshold() {
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let s = DataArray::from_vec("val", vec![0.0f64, 1.0, 2.0, 3.0], 1);
        pd.point_data_mut().add_array(AnyDataArray::F64(s));

        let mut sel = Selection::new();
        sel.add_node(SelectionNode::from_threshold(
            "val",
            0.0,
            1.5,
            SelectionFieldType::Point,
        ));

        let result = extract_selection(&pd, &sel);
        assert_eq!(result.points.len(), 2); // points 0 and 1
    }

    #[test]
    fn empty_selection() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sel = Selection::new();
        let result = extract_selection(&pd, &sel);
        assert_eq!(result.points.len(), 3); // no selection = clone
    }
}
