use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::Scalar;

/// Thicken a surface mesh into a solid by extruding along normals.
///
/// Creates inner and outer surfaces offset by `thickness/2` along
/// vertex normals, then connects them with side faces at boundaries.
/// Returns a closed solid mesh.
pub fn thicken(input: &PolyData, thickness: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let half = thickness * 0.5;

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 || !valid_cell(cell, n) {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }
    for nm in &mut vnormals {
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }
    }

    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Outer surface (original indices 0..n)
    for i in 0..n {
        let p = input.points.get(i);
        out_pts.push([
            p[0] + vnormals[i][0] * half,
            p[1] + vnormals[i][1] * half,
            p[2] + vnormals[i][2] * half,
        ]);
    }

    // Inner surface (indices n..2n)
    for i in 0..n {
        let p = input.points.get(i);
        out_pts.push([
            p[0] - vnormals[i][0] * half,
            p[1] - vnormals[i][1] * half,
            p[2] - vnormals[i][2] * half,
        ]);
    }

    // Outer faces (same winding)
    for cell in input.polys.iter() {
        if valid_cell(cell, n) {
            out_polys.push_cell(cell);
        }
    }

    // Inner faces (reversed winding)
    for cell in input.polys.iter() {
        if !valid_cell(cell, n) {
            continue;
        }
        let mut rev: Vec<i64> = cell.iter().map(|&id| id + n as i64).collect();
        rev.reverse();
        out_polys.push_cell(&rev);
    }

    // Side faces at boundary edges
    let mut edge_count = std::collections::HashMap::new();
    for cell in input.polys.iter() {
        if !valid_cell(cell, n) {
            continue;
        }
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0usize) += 1;
        }
    }
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            // Boundary edge: create quad connecting outer and inner
            out_polys.push_cell(&[a, b, b + n as i64, a + n as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    duplicate_point_data(input, &mut pd);
    pd
}

fn valid_cell(cell: &[i64], number_of_points: usize) -> bool {
    cell.iter()
        .all(|&id| id >= 0 && (id as usize) < number_of_points)
}

fn duplicate_point_data(input: &PolyData, output: &mut PolyData) {
    let n = input.points.len();
    let input_point_data = input.point_data();

    let active_scalars = input_point_data
        .scalars()
        .map(|array| array.name().to_string());
    let active_vectors = input_point_data
        .vectors()
        .map(|array| array.name().to_string());
    let active_normals = input_point_data
        .normals()
        .map(|array| array.name().to_string());
    let active_tcoords = input_point_data
        .tcoords()
        .map(|array| array.name().to_string());
    let active_tensors = input_point_data
        .tensors()
        .map(|array| array.name().to_string());
    let active_global_ids = input_point_data
        .global_ids()
        .map(|array| array.name().to_string());
    let active_pedigree_ids = input_point_data
        .pedigree_ids()
        .map(|array| array.name().to_string());
    let active_edge_flags = input_point_data
        .edge_flags()
        .map(|array| array.name().to_string());
    let active_tangents = input_point_data
        .tangents()
        .map(|array| array.name().to_string());
    let active_rational_weights = input_point_data
        .rational_weights()
        .map(|array| array.name().to_string());
    let active_higher_order_degrees = input_point_data
        .higher_order_degrees()
        .map(|array| array.name().to_string());
    let active_process_ids = input_point_data
        .process_ids()
        .map(|array| array.name().to_string());

    for ai in 0..input_point_data.num_arrays() {
        if let Some(array) = input_point_data.get_array_by_index(ai) {
            if array.num_tuples() != n {
                continue;
            }
            let point_ids: Vec<usize> = (0..n).chain(0..n).collect();
            output
                .point_data_mut()
                .add_array(copy_array_tuples(array, &point_ids));
        }
    }

    let output_point_data = output.point_data_mut();
    if let Some(name) = active_scalars {
        output_point_data.set_active_scalars(&name);
    }
    if let Some(name) = active_vectors {
        output_point_data.set_active_vectors(&name);
    }
    if let Some(name) = active_normals {
        output_point_data.set_active_normals(&name);
    }
    if let Some(name) = active_tcoords {
        output_point_data.set_active_tcoords(&name);
    }
    if let Some(name) = active_tensors {
        output_point_data.set_active_tensors(&name);
    }
    if let Some(name) = active_global_ids {
        output_point_data.set_active_global_ids(&name);
    }
    if let Some(name) = active_pedigree_ids {
        output_point_data.set_active_pedigree_ids(&name);
    }
    if let Some(name) = active_edge_flags {
        output_point_data.set_active_edge_flags(&name);
    }
    if let Some(name) = active_tangents {
        output_point_data.set_active_tangents(&name);
    }
    if let Some(name) = active_rational_weights {
        output_point_data.set_active_rational_weights(&name);
    }
    if let Some(name) = active_higher_order_degrees {
        output_point_data.set_active_higher_order_degrees(&name);
    }
    if let Some(name) = active_process_ids {
        output_point_data.set_active_process_ids(&name);
    }
}

fn copy_array_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy_typed {
        ($arr:expr, $variant:path) => {{
            $variant(copy_typed_array($arr, tuple_ids))
        }};
    }

    match array {
        AnyDataArray::F32(a) => copy_typed!(a, AnyDataArray::F32),
        AnyDataArray::F64(a) => copy_typed!(a, AnyDataArray::F64),
        AnyDataArray::I8(a) => copy_typed!(a, AnyDataArray::I8),
        AnyDataArray::I16(a) => copy_typed!(a, AnyDataArray::I16),
        AnyDataArray::I32(a) => copy_typed!(a, AnyDataArray::I32),
        AnyDataArray::I64(a) => copy_typed!(a, AnyDataArray::I64),
        AnyDataArray::U8(a) => copy_typed!(a, AnyDataArray::U8),
        AnyDataArray::U16(a) => copy_typed!(a, AnyDataArray::U16),
        AnyDataArray::U32(a) => copy_typed!(a, AnyDataArray::U32),
        AnyDataArray::U64(a) => copy_typed!(a, AnyDataArray::U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, tuple_ids: &[usize]) -> DataArray<T> {
    let mut output = DataArray::new(array.name(), array.num_components());
    for &tuple_id in tuple_ids {
        output.push_tuple(array.tuple(tuple_id));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thicken_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = thicken(&pd, 0.2);
        assert_eq!(result.points.len(), 6); // 3 outer + 3 inner
        assert!(result.polys.num_cells() >= 2); // outer + inner + sides
    }

    #[test]
    fn closed_quad_thicken() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = thicken(&pd, 0.5);
        assert_eq!(result.points.len(), 8);
    }

    #[test]
    fn zero_thickness() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = thicken(&pd, 0.0);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = thicken(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn point_data_type_is_preserved_and_duplicated() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "ids",
                vec![3i32, 5i32, 7i32],
                1,
            )));

        let result = thicken(&pd, 0.2);
        match result.point_data().get_array("ids").unwrap() {
            AnyDataArray::I32(array) => {
                assert_eq!(array.num_tuples(), 6);
                assert_eq!(array.tuple(0), &[3i32]);
                assert_eq!(array.tuple(3), &[3i32]);
            }
            other => panic!("unexpected array type: {:?}", other.scalar_type()),
        }
    }
}
