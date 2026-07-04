use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Remove points that are not referenced by any cell (polys, lines, verts, strips)
/// and reindex the remaining points and cell connectivity.
pub fn remove_unused_points(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    let mut used = vec![false; n];

    // Mark all referenced points
    if !mark_used(&input.polys, &mut used)
        || !mark_used(&input.lines, &mut used)
        || !mark_used(&input.verts, &mut used)
        || !mark_used(&input.strips, &mut used)
    {
        return PolyData::new();
    }

    if used.iter().all(|&is_used| is_used) {
        return input.clone();
    }

    // Build old-to-new index mapping; -1 means unused
    let mut old_to_new: Vec<i64> = vec![-1; n];
    let mut new_idx: i64 = 0;
    let mut out_points = Points::<f64>::new();

    for i in 0..n {
        if used[i] {
            old_to_new[i] = new_idx;
            out_points.push(input.points.get(i));
            new_idx += 1;
        }
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd.polys = remap_cell_array(&input.polys, &old_to_new);
    pd.lines = remap_cell_array(&input.lines, &old_to_new);
    pd.verts = remap_cell_array(&input.verts, &old_to_new);
    pd.strips = remap_cell_array(&input.strips, &old_to_new);
    remap_point_data(input, &used, &mut pd);
    pd
}

fn mark_used(cells: &CellArray, used: &mut [bool]) -> bool {
    for cell in cells.iter() {
        for &id in cell {
            if id < 0 || id as usize >= used.len() {
                return false;
            }
            used[id as usize] = true;
        }
    }
    true
}

fn remap_cell_array(src: &CellArray, old_to_new: &[i64]) -> CellArray {
    let mut dst = CellArray::new();
    for cell in src.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| old_to_new[id as usize]).collect();
        dst.push_cell(&remapped);
    }
    dst
}

fn remap_point_data(input: &PolyData, used: &[bool], output: &mut PolyData) {
    output.point_data_mut().clear();

    for array in input.point_data().iter() {
        if array.num_tuples() == used.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, used));
        }
    }

    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn select_tuples(array: &AnyDataArray, used: &[bool]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for (tuple_id, &is_used) in used.iter().enumerate() {
                if is_used {
                    out.push_tuple($array.tuple(tuple_id));
                }
            }
            AnyDataArray::$variant(out)
        }};
    }

    match array {
        AnyDataArray::F32(a) => select!(a, F32),
        AnyDataArray::F64(a) => select!(a, F64),
        AnyDataArray::I8(a) => select!(a, I8),
        AnyDataArray::I16(a) => select!(a, I16),
        AnyDataArray::I32(a) => select!(a, I32),
        AnyDataArray::I64(a) => select!(a, I64),
        AnyDataArray::U8(a) => select!(a, U8),
        AnyDataArray::U16(a) => select!(a, U16),
        AnyDataArray::U32(a) => select!(a, U32),
        AnyDataArray::U64(a) => select!(a, U64),
    }
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_unreferenced_points() {
        // 4 points but triangle only uses 0,1,2 — point 3 is unused
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([99.0, 99.0, 99.0]); // unused
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn keeps_all_when_all_used() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn reindexes_correctly() {
        // Points 0,1 unused; triangle uses points 2,3,4
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // unused
        pd.points.push([0.0, 0.0, 0.0]); // unused
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
        // First point in result should be old point 2 => (1,0,0)
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10);
        // Cell should now reference 0,1,2
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert_eq!(cells[0], vec![0, 1, 2]);
    }

    #[test]
    fn preserves_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([99.0, 99.0, 99.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.strips.num_cells(), 1);
        let strips: Vec<Vec<i64>> = result.strips.iter().map(|c| c.to_vec()).collect();
        assert_eq!(strips[0], vec![0, 1, 2, 3]);
    }

    #[test]
    fn invalid_point_id_returns_empty_output() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 0]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.total_cells(), 0);
    }

    #[test]
    fn remaps_point_data_for_surviving_points() {
        use crate::data::DataArray;

        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[1, 2, 1]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("id");

        let result = remove_unused_points(&pd);
        let values = result.point_data().get_array("id").unwrap().to_f64_vec();
        assert_eq!(values, vec![20.0, 30.0]);
        assert_eq!(result.point_data().scalars().unwrap().name(), "id");
    }
}
