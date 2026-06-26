use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Subsample points from a PolyData by keeping every Nth point.
///
/// The output contains only vertex cells (one per kept point).
/// Useful for reducing point clouds before glyphing.
pub fn mask_points(input: &PolyData, every_nth: usize) -> PolyData {
    let every_nth = every_nth.max(1);
    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut selected_ids = Vec::new();

    let mut idx = 0;
    let mut i = 0;
    while i < input.points.len() {
        out_points.push(input.points.get(i));
        out_verts.push_cell(&[idx]);
        selected_ids.push(i);
        idx += 1;
        i += every_nth;
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    copy_selected_point_data(input, &mut pd, &selected_ids);
    pd
}

/// Subsample points randomly using a deterministic seed.
///
/// Keeps approximately `ratio` fraction of points (0.0–1.0).
pub fn mask_points_random(input: &PolyData, ratio: f64, seed: u64) -> PolyData {
    let ratio = ratio.clamp(0.0, 1.0);
    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut selected_ids = Vec::new();

    // Simple deterministic hash-based selection
    let mut state = seed;
    let mut idx = 0i64;
    for i in 0..input.points.len() {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let r = (state & 0xFFFFFFFF) as f64 / 0xFFFFFFFF_u64 as f64;
        if r < ratio {
            out_points.push(input.points.get(i));
            out_verts.push_cell(&[idx]);
            selected_ids.push(i);
            idx += 1;
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    copy_selected_point_data(input, &mut pd, &selected_ids);
    pd
}

fn copy_selected_point_data(input: &PolyData, output: &mut PolyData, selected_ids: &[usize]) {
    for i in 0..input.point_data().num_arrays() {
        if let Some(array) = input.point_data().get_array_by_index(i) {
            if array.num_tuples() == input.points.len() {
                output
                    .point_data_mut()
                    .add_array(subset_any_array(array, selected_ids));
            }
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn subset_any_array(array: &AnyDataArray, selected_ids: &[usize]) -> AnyDataArray {
    match array {
        AnyDataArray::F32(a) => AnyDataArray::F32(subset_data_array(a, selected_ids)),
        AnyDataArray::F64(a) => AnyDataArray::F64(subset_data_array(a, selected_ids)),
        AnyDataArray::I8(a) => AnyDataArray::I8(subset_data_array(a, selected_ids)),
        AnyDataArray::I16(a) => AnyDataArray::I16(subset_data_array(a, selected_ids)),
        AnyDataArray::I32(a) => AnyDataArray::I32(subset_data_array(a, selected_ids)),
        AnyDataArray::I64(a) => AnyDataArray::I64(subset_data_array(a, selected_ids)),
        AnyDataArray::U8(a) => AnyDataArray::U8(subset_data_array(a, selected_ids)),
        AnyDataArray::U16(a) => AnyDataArray::U16(subset_data_array(a, selected_ids)),
        AnyDataArray::U32(a) => AnyDataArray::U32(subset_data_array(a, selected_ids)),
        AnyDataArray::U64(a) => AnyDataArray::U64(subset_data_array(a, selected_ids)),
    }
}

fn subset_data_array<T: Scalar>(array: &DataArray<T>, selected_ids: &[usize]) -> DataArray<T> {
    let mut result = DataArray::new(array.name(), array.num_components());
    for &id in selected_ids {
        result.push_tuple(array.tuple(id));
    }
    result
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
    fn every_second_point() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let result = mask_points(&pd, 2);
        assert_eq!(result.points.len(), 5); // 0, 2, 4, 6, 8
        assert_eq!(result.verts.num_cells(), 5);
        let p = result.points.get(2);
        assert!((p[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn every_point() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let result = mask_points(&pd, 1);
        assert_eq!(result.points.len(), 5);
    }

    #[test]
    fn preserves_point_data_for_striding() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "ids",
                vec![10, 11, 12, 13, 14],
                1,
            )));

        let result = mask_points(&pd, 2);
        let ids = result.point_data().get_array("ids").unwrap();
        let mut value = [0.0];
        ids.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 10.0);
        ids.tuple_as_f64(1, &mut value);
        assert_eq!(value[0], 12.0);
        ids.tuple_as_f64(2, &mut value);
        assert_eq!(value[0], 14.0);
    }

    #[test]
    fn random_mask() {
        let mut pd = PolyData::new();
        for i in 0..100 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let result = mask_points_random(&pd, 0.5, 42);
        // Should keep roughly half
        assert!(result.points.len() > 20);
        assert!(result.points.len() < 80);
    }
}
