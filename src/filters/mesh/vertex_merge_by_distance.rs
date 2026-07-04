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
    let mut out_pts = Points::<f64>::new();
    let mut representatives = Vec::new();

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

    let mut kept_cell_ids = Vec::new();
    let out_verts = remap_cells(&input.verts, &remap, 1, 0, &mut kept_cell_ids);
    let line_offset = input.verts.num_cells();
    let out_lines = remap_cells(&input.lines, &remap, 2, line_offset, &mut kept_cell_ids);
    let poly_offset = line_offset + input.lines.num_cells();
    let out_polys = remap_cells(&input.polys, &remap, 3, poly_offset, &mut kept_cell_ids);
    let strip_offset = poly_offset + input.polys.num_cells();
    let out_strips = remap_cells(&input.strips, &remap, 4, strip_offset, &mut kept_cell_ids);

    let mut pd = input.clone();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd.strips = out_strips;
    remap_point_data(input, &representatives, &mut pd);
    remap_cell_data(input, &kept_cell_ids, &mut pd);
    pd
}

fn remap_cells(
    cells: &CellArray,
    remap: &[usize],
    minimum_size: usize,
    cell_offset: usize,
    kept_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let Some(mapped) = remap_cell(cell, remap) else {
            continue;
        };
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= minimum_size {
            out.push_cell(&mapped);
            kept_cell_ids.push(cell_offset + cell_id);
        }
    }
    out
}

fn remap_cell(cell: &[i64], remap: &[usize]) -> Option<Vec<i64>> {
    let mut mapped = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || id as usize >= remap.len() {
            return None;
        }
        mapped.push(remap[id as usize] as i64);
    }
    Some(mapped)
}

fn remap_point_data(input: &PolyData, old_point_ids: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, old_point_ids));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn remap_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    output.cell_data_mut().clear();
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_tuples(array: &AnyDataArray, old_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(select_typed_tuples($array, old_ids))
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

fn select_typed_tuples<T: Scalar>(array: &DataArray<T>, old_ids: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(old_ids.len() * num_components);
    for &old_id in old_ids {
        data.extend_from_slice(array.tuple(old_id));
    }
    DataArray::from_vec(array.name(), data, num_components)
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
    fn preserves_representative_point_and_cell_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.001, 0.0, 0.0]);
        pd.lines.push_cell(&[4, 5]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
                1,
            )));
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("ids", vec![7, 9], 1)));

        let result = merge_close_vertices(&pd, 0.01);

        let temperature = result.point_data().get_array("temperature").unwrap();
        let mut scalar = [0.0f64];
        temperature.tuple_as_f64(0, &mut scalar);
        assert_eq!(scalar[0], 10.0);

        let ids = result.cell_data().get_array("ids").unwrap();
        assert_eq!(ids.num_tuples(), 1);
        ids.tuple_as_f64(0, &mut scalar);
        assert_eq!(scalar[0], 9.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(merge_close_vertices(&pd, 0.1).points.len(), 0);
    }

    #[test]
    fn invalid_distance_is_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 1]);

        assert_eq!(merge_close_vertices(&pd, -1.0).points.len(), 2);
        assert_eq!(merge_close_vertices(&pd, f64::NAN).points.len(), 2);
    }

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = merge_close_vertices(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
