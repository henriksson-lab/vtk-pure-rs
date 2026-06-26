use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Extract points by index from a PolyData, producing vertex cells.
pub fn extract_points(input: &PolyData, point_indices: &[usize]) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for (new_idx, &pi) in point_indices.iter().enumerate() {
        if pi < input.points.len() {
            out_points.push(input.points.get(pi));
            out_verts.push_cell(&[new_idx as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    copy_point_data(input, point_indices, &mut pd);
    pd
}

/// Extract points where a scalar condition is met.
pub fn extract_points_by_scalar(
    input: &PolyData,
    scalar_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
    let arr = match input.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let mut indices = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples().min(input.points.len()) {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= min_val && buf[0] <= max_val {
            indices.push(i);
        }
    }

    extract_points(input, &indices)
}

fn copy_point_data(input: &PolyData, point_indices: &[usize], output: &mut PolyData) {
    let valid_indices: Vec<usize> = point_indices
        .iter()
        .copied()
        .filter(|&pi| pi < input.points.len())
        .collect();

    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, &valid_indices));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
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
    use crate::data::DataArray;

    #[test]
    fn extract_by_index() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);

        let result = extract_points(&pd, &[1, 3]);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.verts.num_cells(), 2);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn extract_by_scalar() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let scalars = DataArray::from_vec("temp", vec![10.0, 20.0, 30.0, 40.0, 50.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = extract_points_by_scalar(&pd, "temp", 25.0, 45.0);
        assert_eq!(result.points.len(), 2); // points with temp 30, 40
    }
}
