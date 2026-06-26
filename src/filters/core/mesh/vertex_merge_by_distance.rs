use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSetAttributes, KdTree, Points, PolyData,
};
use crate::types::Scalar;

/// Merge vertices that are within `distance` of each other using k-d tree.
///
/// More efficient than `vertex_glue` for large meshes. Merges to the
/// first-encountered vertex in each cluster.
pub fn merge_close_vertices(input: &PolyData, distance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || !distance.is_finite() || distance < 0.0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);
    let d2 = distance * distance;

    let mut remap = vec![usize::MAX; n];
    let mut representatives = Vec::new();
    let mut out_pts = Points::<f64>::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let idx = out_pts.len();
        out_pts.push(pts[i]);
        representatives.push(i);
        remap[i] = idx;

        let nbrs = tree.find_within_radius(pts[i], distance);
        for &(j, jd2) in &nbrs {
            if j != i && remap[j] == usize::MAX && jd2 <= d2 {
                remap[j] = idx;
            }
        }
    }

    let mut pd = input.clone();
    pd.points = out_pts;
    let mut old_cell_ids = Vec::new();
    let mut old_offset = 0usize;
    pd.verts = remap_cell_array(&input.verts, &remap, 1, old_offset, &mut old_cell_ids);
    old_offset += input.verts.num_cells();
    pd.lines = remap_cell_array(&input.lines, &remap, 2, old_offset, &mut old_cell_ids);
    old_offset += input.lines.num_cells();
    pd.polys = remap_cell_array(&input.polys, &remap, 3, old_offset, &mut old_cell_ids);
    old_offset += input.polys.num_cells();
    pd.strips = remap_cell_array(&input.strips, &remap, 3, old_offset, &mut old_cell_ids);
    remap_point_data(input, &representatives, &mut pd);
    remap_cell_data(input, &old_cell_ids, &mut pd);
    pd
}

fn remap_cell_array(
    cells: &CellArray,
    remap: &[usize],
    min_unique: usize,
    old_offset: usize,
    old_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let mut mapped = Vec::with_capacity(cell.len());
        let mut valid = true;
        for &id in cell {
            if id < 0 || id as usize >= remap.len() || remap[id as usize] == usize::MAX {
                valid = false;
                break;
            }
            mapped.push(remap[id as usize] as i64);
        }
        if !valid {
            continue;
        }

        let mut unique = mapped.clone();
        unique.sort_unstable();
        unique.dedup();
        if unique.len() >= min_unique {
            out.push_cell(&mapped);
            old_cell_ids.push(old_offset + cell_id);
        }
    }
    out
}

fn remap_point_data(input: &PolyData, representatives: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for array in input.point_data().field_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_representative_tuples(array, representatives));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn select_representative_tuples(array: &AnyDataArray, representatives: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in representatives {
                out.push_tuple($array.tuple(tuple_id));
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

fn remap_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    output.cell_data_mut().clear();
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_cell_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_cell_tuples(array: &AnyDataArray, old_cell_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(select_typed_cell_tuples($array, old_cell_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => select!(array, F32),
        AnyDataArray::F64(array) => select!(array, F64),
        AnyDataArray::I8(array) => select!(array, I8),
        AnyDataArray::I16(array) => select!(array, I16),
        AnyDataArray::I32(array) => select!(array, I32),
        AnyDataArray::I64(array) => select!(array, I64),
        AnyDataArray::U8(array) => select!(array, U8),
        AnyDataArray::U16(array) => select!(array, U16),
        AnyDataArray::U32(array) => select!(array, U32),
        AnyDataArray::U64(array) => select!(array, U64),
    }
}

fn select_typed_cell_tuples<T: Scalar>(
    array: &DataArray<T>,
    old_cell_ids: &[usize],
) -> DataArray<T> {
    let mut data = Vec::with_capacity(old_cell_ids.len() * array.num_components());
    for &old_cell_id in old_cell_ids {
        data.extend_from_slice(array.tuple(old_cell_id));
    }
    DataArray::from_vec(array.name(), data, array.num_components())
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
    fn merge_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]); // close to 0
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.001, 0.0, 0.0]); // close to 2
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 2, 4]);
        pd.polys.push_cell(&[1, 3, 4]);

        let result = merge_close_vertices(&pd, 0.01);
        assert!(result.points.len() < 5);
    }

    #[test]
    fn no_merge_far() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = merge_close_vertices(&pd, 0.001);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(merge_close_vertices(&pd, 0.1).points.len(), 0);
    }

    #[test]
    fn remaps_cell_data_for_kept_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![7, 9],
                1,
            )));
        pd.cell_data_mut().set_active_scalars("cell_id");

        let result = merge_close_vertices(&pd, 0.01);

        assert_eq!(result.polys.num_cells(), 1);
        let cell_id = result.cell_data().get_array("cell_id").unwrap();
        assert_eq!(cell_id.num_tuples(), 1);
        let mut value = [0.0];
        cell_id.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 9.0);
        assert!(result.cell_data().scalars().is_some());
    }

    #[test]
    fn preserves_active_point_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "value",
                vec![2.0, 4.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("value");

        let result = merge_close_vertices(&pd, 0.01);

        assert_eq!(result.points.len(), 1);
        assert!(result.point_data().scalars().is_some());
    }
}
