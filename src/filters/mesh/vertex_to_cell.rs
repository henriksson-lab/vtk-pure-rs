use crate::data::{AnyDataArray, DataArray, PolyData};

/// Convert vertex selection (binary mask) to cell selection.
///
/// A cell is selected if ALL its vertices are selected (mask >= threshold).
/// Adds "CellSelected" cell data.
pub fn vertex_mask_to_cell_mask(input: &PolyData, mask_name: &str, threshold: f64) -> PolyData {
    let arr = match input.point_data().get_array(mask_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = input.points.len();
    let mut buf = [0.0f64];
    let selected: Vec<bool> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0] >= threshold
            } else {
                false
            }
        })
        .collect();

    let mut cell_mask = Vec::with_capacity(input.total_cells());
    push_cell_mask(&input.verts, &selected, &mut cell_mask);
    push_cell_mask(&input.lines, &selected, &mut cell_mask);
    push_cell_mask(&input.polys, &selected, &mut cell_mask);
    push_cell_mask(&input.strips, &selected, &mut cell_mask);

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CellSelected",
            cell_mask,
            1,
        )));
    pd
}

/// Convert cell selection to vertex selection.
///
/// A vertex is selected if ANY of its adjacent cells is selected.
pub fn cell_mask_to_vertex_mask(input: &PolyData, mask_name: &str, threshold: f64) -> PolyData {
    let arr = match input.cell_data().get_array(mask_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = input.points.len();
    let mut buf = [0.0f64];
    let mut vertex_selected = vec![0.0f64; n];
    let mut cell_idx = 0;

    select_vertices_from_cells(
        &input.verts,
        n,
        arr,
        threshold,
        &mut cell_idx,
        &mut vertex_selected,
        &mut buf,
    );
    select_vertices_from_cells(
        &input.lines,
        n,
        arr,
        threshold,
        &mut cell_idx,
        &mut vertex_selected,
        &mut buf,
    );
    select_vertices_from_cells(
        &input.polys,
        n,
        arr,
        threshold,
        &mut cell_idx,
        &mut vertex_selected,
        &mut buf,
    );
    select_vertices_from_cells(
        &input.strips,
        n,
        arr,
        threshold,
        &mut cell_idx,
        &mut vertex_selected,
        &mut buf,
    );

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexSelected",
            vertex_selected,
            1,
        )));
    pd
}

fn push_cell_mask(source: &crate::data::CellArray, selected: &[bool], cell_mask: &mut Vec<f64>) {
    for cell in source.iter() {
        let all_selected = !cell.is_empty()
            && cell.iter().all(|&id| {
                usize::try_from(id)
                    .ok()
                    .and_then(|idx| selected.get(idx))
                    .copied()
                    .unwrap_or(false)
            });
        cell_mask.push(if all_selected { 1.0 } else { 0.0 });
    }
}

fn select_vertices_from_cells(
    source: &crate::data::CellArray,
    num_points: usize,
    array: &crate::data::AnyDataArray,
    threshold: f64,
    cell_idx: &mut usize,
    vertex_selected: &mut [f64],
    tuple: &mut [f64],
) {
    for cell in source.iter() {
        if *cell_idx < array.num_tuples() {
            array.tuple_as_f64(*cell_idx, tuple);
            if tuple[0] >= threshold {
                for &id in cell.iter() {
                    let Ok(id) = usize::try_from(id) else {
                        continue;
                    };
                    if id < num_points {
                        vertex_selected[id] = 1.0;
                    }
                }
            }
        }
        *cell_idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_to_cell() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![1.0, 1.0, 1.0, 0.0],
                1,
            )));

        let result = vertex_mask_to_cell_mask(&pd, "mask", 0.5);
        let arr = result.cell_data().get_array("CellSelected").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // all 3 selected
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0); // vertex 3 not selected
    }

    #[test]
    fn cell_to_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "sel",
                vec![1.0, 0.0],
                1,
            )));

        let result = cell_mask_to_vertex_mask(&pd, "sel", 0.5);
        let arr = result.point_data().get_array("VertexSelected").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // in cell 0
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 0.0); // only in cell 1
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let r = vertex_mask_to_cell_mask(&pd, "nope", 0.5);
        assert_eq!(r.points.len(), 0);
    }
}
