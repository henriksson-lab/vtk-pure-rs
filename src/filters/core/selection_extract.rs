use crate::data::{PolyData, Selection, SelectionContentType, SelectionFieldType};

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
            let indices: Vec<usize> = node.selection_list.iter().map(|&v| v as usize).collect();
            match node.field_type {
                SelectionFieldType::Point => extract_by_point_indices(input, &indices),
                SelectionFieldType::Cell => extract_by_cell_indices(input, &indices),
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
            }
        }
        _ => input.clone(),
    }
}

fn extract_by_point_indices(input: &PolyData, indices: &[usize]) -> PolyData {
    let mut pd = PolyData::new();
    let n = input.points.len();

    // Map old point index → new point index
    let mut old_to_new = vec![usize::MAX; n];
    for (new_idx, &old_idx) in indices.iter().enumerate() {
        if old_idx < n {
            pd.points.push(input.points.get(old_idx));
            old_to_new[old_idx] = new_idx;
        }
    }

    // Copy cells that only reference selected points
    for cell in input.polys.iter() {
        if cell
            .iter()
            .all(|&pid| (pid as usize) < n && old_to_new[pid as usize] != usize::MAX)
        {
            let new_cell: Vec<i64> = cell
                .iter()
                .map(|&pid| old_to_new[pid as usize] as i64)
                .collect();
            pd.polys.push_cell(&new_cell);
        }
    }

    pd
}

fn extract_by_cell_indices(input: &PolyData, indices: &[usize]) -> PolyData {
    let mut pd = PolyData::new();
    let n = input.points.len();
    let mut point_used = vec![false; n];

    // Mark which points are used by selected cells
    let all_cells: Vec<&[i64]> = input.polys.iter().collect();
    for &ci in indices {
        if ci < all_cells.len() {
            for &pid in all_cells[ci] {
                if (pid as usize) < n {
                    point_used[pid as usize] = true;
                }
            }
        }
    }

    // Build point mapping
    let mut old_to_new = vec![usize::MAX; n];
    for (i, &used) in point_used.iter().enumerate() {
        if used {
            old_to_new[i] = pd.points.len();
            pd.points.push(input.points.get(i));
        }
    }

    // Copy selected cells with remapped indices
    for &ci in indices {
        if ci < all_cells.len() {
            let new_cell: Vec<i64> = all_cells[ci]
                .iter()
                .map(|&pid| old_to_new[pid as usize] as i64)
                .collect();
            pd.polys.push_cell(&new_cell);
        }
    }

    pd
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
