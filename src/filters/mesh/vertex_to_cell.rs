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
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= threshold
        })
        .collect();

    let mut cell_mask = Vec::new();
    for cell in input.polys.iter() {
        cell_mask.push(if cell.iter().all(|&id| selected[id as usize]) {
            1.0
        } else {
            0.0
        });
    }

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

    for (ci, cell) in input.polys.iter().enumerate() {
        arr.tuple_as_f64(ci, &mut buf);
        if buf[0] >= threshold {
            for &id in cell.iter() {
                vertex_selected[id as usize] = 1.0;
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexSelected",
            vertex_selected,
            1,
        )));
    pd
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
