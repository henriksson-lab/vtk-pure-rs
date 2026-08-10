use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Extract vertices as a point cloud PolyData (strip all cells).
///
/// Creates vertex cells for every point and removes polygon/line/strip cells.
pub fn vertices_only(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut verts = CellArray::new();
    for i in 0..n {
        verts.push_cell(&[i as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.verts = verts;
    *pd.point_data_mut() = input.point_data().clone();
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

/// Extract cell centroids as a point cloud.
///
/// Each polygon becomes a single point at its centroid.
/// Cell data becomes point data on the output.
pub fn cell_centroids_as_points(input: &PolyData) -> PolyData {
    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut old_cell_ids = Vec::new();
    let mut cell_idx = 0;

    push_cell_centroids(
        &input.verts,
        input,
        &mut cell_idx,
        &mut out_pts,
        &mut out_verts,
        &mut old_cell_ids,
    );
    push_cell_centroids(
        &input.lines,
        input,
        &mut cell_idx,
        &mut out_pts,
        &mut out_verts,
        &mut old_cell_ids,
    );
    push_cell_centroids(
        &input.polys,
        input,
        &mut cell_idx,
        &mut out_pts,
        &mut out_verts,
        &mut old_cell_ids,
    );
    push_cell_centroids(
        &input.strips,
        input,
        &mut cell_idx,
        &mut out_pts,
        &mut out_verts,
        &mut old_cell_ids,
    );

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    copy_cell_data_to_point_data(input, &old_cell_ids, &mut pd);
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

/// Extract edge midpoints as a point cloud.
///
/// Re-exported from [`crate::filters::mesh::mesh_sample_edges`], which holds the
/// single implementation.
pub use crate::filters::mesh::mesh_sample_edges::edge_midpoints;

fn push_cell_centroids(
    cells: &CellArray,
    input: &PolyData,
    cell_idx: &mut usize,
    out_pts: &mut Points<f64>,
    out_verts: &mut CellArray,
    old_cell_ids: &mut Vec<usize>,
) {
    for cell in cells.iter() {
        if let Some(centroid) = cell_centroid(cell, input) {
            let idx = out_pts.len() as i64;
            out_pts.push(centroid);
            out_verts.push_cell(&[idx]);
            old_cell_ids.push(*cell_idx);
        }
        *cell_idx += 1;
    }
}

fn cell_centroid(cell: &[i64], input: &PolyData) -> Option<[f64; 3]> {
    if cell.is_empty() {
        return None;
    }
    let mut c = [0.0; 3];
    let mut count = 0usize;
    for &id in cell {
        let Ok(id) = usize::try_from(id) else {
            continue;
        };
        if id >= input.points.len() {
            continue;
        }
        let p = input.points.get(id);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
        count += 1;
    }
    if count == 0 {
        return None;
    }
    let n = count as f64;
    Some([c[0] / n, c[1] / n, c[2] / n])
}

fn copy_cell_data_to_point_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .point_data_mut()
                .add_array(remap_array(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.point_data_mut());
}

fn remap_array(array: &AnyDataArray, old_tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($arr:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($arr, old_tuple_ids))
        };
    }
    match array {
        AnyDataArray::F32(a) => remap!(a, F32),
        AnyDataArray::F64(a) => remap!(a, F64),
        AnyDataArray::I8(a) => remap!(a, I8),
        AnyDataArray::I16(a) => remap!(a, I16),
        AnyDataArray::I32(a) => remap!(a, I32),
        AnyDataArray::I64(a) => remap!(a, I64),
        AnyDataArray::U8(a) => remap!(a, U8),
        AnyDataArray::U16(a) => remap!(a, U16),
        AnyDataArray::U32(a) => remap!(a, U32),
        AnyDataArray::U64(a) => remap!(a, U64),
    }
}

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_tuple_ids: &[usize]) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(old_tuple_ids.len() * nc);
    for &old_id in old_tuple_ids {
        data.extend_from_slice(array.tuple(old_id));
    }
    DataArray::from_vec(array.name(), data, nc)
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
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn vertices_only_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertices_only(&pd);
        assert_eq!(result.verts.num_cells(), 3);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn cell_centroids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([0.0, 3.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cell_centroids_as_points(&pd);
        assert_eq!(result.points.len(), 1);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn preserves_point_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vec![42.0], 1)));

        let result = vertices_only(&pd);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(vertices_only(&pd).points.len(), 0);
    }
}
