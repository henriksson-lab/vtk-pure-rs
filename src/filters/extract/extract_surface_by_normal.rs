use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Extract triangles whose face normal points in a given direction.
///
/// Keeps cells whose face normal has a dot product with `direction`
/// above `threshold` (range -1 to 1). Useful for extracting top/bottom/side faces.
pub fn extract_surface_by_normal(
    input: &PolyData,
    direction: [f64; 3],
    threshold: f64,
) -> PolyData {
    let dlen =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dlen < 1e-15 {
        return PolyData::new();
    }
    let dir = [
        direction[0] / dlen,
        direction[1] / dlen,
        direction[2] / dlen,
    ];

    let mut kept_pts = std::collections::HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut old_point_ids = Vec::new();
    let mut old_cell_ids = Vec::new();
    let polys_offset = input.verts.num_cells() + input.lines.num_cells();

    for (cell_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }

        // Compute face normal from first triangle
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let nlen = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nlen < 1e-15 {
            continue;
        }

        let dot = (n[0] * dir[0] + n[1] * dir[1] + n[2] * dir[2]) / nlen;
        if dot >= threshold {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *kept_pts.entry(id).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        old_point_ids.push(id as usize);
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
            old_cell_ids.push(polys_offset + cell_id);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    copy_point_data(input, &old_point_ids, &mut pd);
    copy_cell_data(input, &old_cell_ids, &mut pd);
    pd
}

fn copy_point_data(input: &PolyData, old_point_ids: &[usize], output: &mut PolyData) {
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, old_point_ids));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn copy_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
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

    #[test]
    fn extract_upward_faces() {
        let mut pd = PolyData::new();
        // Upward-facing triangle (normal +Z)
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        // Downward-facing triangle (normal -Z)
        pd.points.push([0.0, 0.0, 1.0]);
        pd.points.push([0.0, 1.0, 1.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = extract_surface_by_normal(&pd, [0.0, 0.0, 1.0], 0.5);
        assert_eq!(result.polys.num_cells(), 1); // only upward
    }

    #[test]
    fn extract_all_with_low_threshold() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = extract_surface_by_normal(&pd, [0.0, 0.0, 1.0], -1.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = extract_surface_by_normal(&pd, [0.0, 1.0, 0.0], 0.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
