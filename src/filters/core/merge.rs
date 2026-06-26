use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};

/// Merge multiple PolyData meshes into a single PolyData.
///
/// Point indices in cells are renumbered to account for the offset of
/// each input's points in the combined output.
pub fn merge(inputs: &[&PolyData]) -> PolyData {
    if inputs.is_empty() {
        return PolyData::new();
    }
    if inputs.len() == 1 {
        return inputs[0].clone();
    }

    let mut result = PolyData::new();
    let datasets: Vec<&PolyData> = inputs
        .iter()
        .copied()
        .filter(|pd| !pd.points.is_empty())
        .collect();
    if datasets.is_empty() {
        return result;
    }

    for pd in &datasets {
        let base = result.points.len() as i64;

        // Copy points
        for p in &pd.points {
            result.points.push(p);
        }

        // Copy polys with offset
        for cell in pd.polys.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.polys.push_cell(&offset_cell);
        }

        // Copy lines with offset
        for cell in pd.lines.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.lines.push_cell(&offset_cell);
        }

        // Copy verts with offset
        for cell in pd.verts.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.verts.push_cell(&offset_cell);
        }

        // Copy strips with offset
        for cell in pd.strips.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.strips.push_cell(&offset_cell);
        }
    }

    copy_common_point_data(&datasets, &mut result);
    copy_common_cell_data(&datasets, &mut result);

    result
}

fn copy_common_point_data(inputs: &[&PolyData], output: &mut PolyData) {
    if inputs.is_empty() {
        return;
    }

    for array in inputs[0].point_data().iter() {
        let name = array.name();
        if inputs.iter().all(|pd| {
            pd.point_data().get_array(name).is_some_and(|other| {
                arrays_compatible(array, other) && other.num_tuples() == pd.points.len()
            })
        }) {
            if let Some(appended) = append_point_array(name, array, inputs) {
                output.point_data_mut().add_array(appended);
            }
        }
    }
    copy_active_attributes(inputs[0].point_data(), output.point_data_mut());
}

fn copy_common_cell_data(inputs: &[&PolyData], output: &mut PolyData) {
    if inputs.is_empty() {
        return;
    }

    for array in inputs[0].cell_data().iter() {
        let name = array.name();
        if inputs.iter().all(|pd| {
            pd.cell_data().get_array(name).is_some_and(|other| {
                arrays_compatible(array, other) && other.num_tuples() == pd.total_cells()
            })
        }) {
            if let Some(appended) = append_cell_array(name, array, inputs) {
                output.cell_data_mut().add_array(appended);
            }
        }
    }
    copy_active_attributes(inputs[0].cell_data(), output.cell_data_mut());
}

fn arrays_compatible(a: &AnyDataArray, b: &AnyDataArray) -> bool {
    a.scalar_type() == b.scalar_type() && a.num_components() == b.num_components()
}

fn append_point_array(
    name: &str,
    template: &AnyDataArray,
    inputs: &[&PolyData],
) -> Option<AnyDataArray> {
    macro_rules! append {
        ($variant:ident, $ty:ty) => {{
            let mut out = DataArray::<$ty>::new(name, template.num_components());
            for pd in inputs {
                let Some(AnyDataArray::$variant(array)) = pd.point_data().get_array(name) else {
                    return None;
                };
                for tuple in array.iter_tuples() {
                    out.push_tuple(tuple);
                }
            }
            Some(AnyDataArray::$variant(out))
        }};
    }

    match template {
        AnyDataArray::F32(_) => append!(F32, f32),
        AnyDataArray::F64(_) => append!(F64, f64),
        AnyDataArray::I8(_) => append!(I8, i8),
        AnyDataArray::I16(_) => append!(I16, i16),
        AnyDataArray::I32(_) => append!(I32, i32),
        AnyDataArray::I64(_) => append!(I64, i64),
        AnyDataArray::U8(_) => append!(U8, u8),
        AnyDataArray::U16(_) => append!(U16, u16),
        AnyDataArray::U32(_) => append!(U32, u32),
        AnyDataArray::U64(_) => append!(U64, u64),
    }
}

fn append_cell_array(
    name: &str,
    template: &AnyDataArray,
    inputs: &[&PolyData],
) -> Option<AnyDataArray> {
    macro_rules! append {
        ($variant:ident, $ty:ty) => {{
            let mut out = DataArray::<$ty>::new(name, template.num_components());
            for category in 0..4 {
                for pd in inputs {
                    let Some(AnyDataArray::$variant(array)) = pd.cell_data().get_array(name) else {
                        return None;
                    };
                    let ranges = cell_tuple_ranges(pd);
                    for tuple_id in ranges[category].clone() {
                        out.push_tuple(array.tuple(tuple_id));
                    }
                }
            }
            Some(AnyDataArray::$variant(out))
        }};
    }

    match template {
        AnyDataArray::F32(_) => append!(F32, f32),
        AnyDataArray::F64(_) => append!(F64, f64),
        AnyDataArray::I8(_) => append!(I8, i8),
        AnyDataArray::I16(_) => append!(I16, i16),
        AnyDataArray::I32(_) => append!(I32, i32),
        AnyDataArray::I64(_) => append!(I64, i64),
        AnyDataArray::U8(_) => append!(U8, u8),
        AnyDataArray::U16(_) => append!(U16, u16),
        AnyDataArray::U32(_) => append!(U32, u32),
        AnyDataArray::U64(_) => append!(U64, u64),
    }
}

fn cell_tuple_ranges(pd: &PolyData) -> [std::ops::Range<usize>; 4] {
    let verts = pd.verts.num_cells();
    let lines = pd.lines.num_cells();
    let polys = pd.polys.num_cells();
    let strips = pd.strips.num_cells();
    [
        0..verts,
        verts..verts + lines,
        verts + lines..verts + lines + polys,
        verts + lines + polys..verts + lines + polys + strips,
    ]
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        if output.has_array(array.name()) {
            output.set_active_scalars(array.name());
        }
    }
    if let Some(array) = input.vectors() {
        if output.has_array(array.name()) {
            output.set_active_vectors(array.name());
        }
    }
    if let Some(array) = input.normals() {
        if output.has_array(array.name()) {
            output.set_active_normals(array.name());
        }
    }
    if let Some(array) = input.tcoords() {
        if output.has_array(array.name()) {
            output.set_active_tcoords(array.name());
        }
    }
    if let Some(array) = input.tensors() {
        if output.has_array(array.name()) {
            output.set_active_tensors(array.name());
        }
    }
    if let Some(array) = input.global_ids() {
        if output.has_array(array.name()) {
            output.set_active_global_ids(array.name());
        }
    }
    if let Some(array) = input.pedigree_ids() {
        if output.has_array(array.name()) {
            output.set_active_pedigree_ids(array.name());
        }
    }
    if let Some(array) = input.edge_flags() {
        if output.has_array(array.name()) {
            output.set_active_edge_flags(array.name());
        }
    }
    if let Some(array) = input.tangents() {
        if output.has_array(array.name()) {
            output.set_active_tangents(array.name());
        }
    }
    if let Some(array) = input.rational_weights() {
        if output.has_array(array.name()) {
            output.set_active_rational_weights(array.name());
        }
    }
    if let Some(array) = input.higher_order_degrees() {
        if output.has_array(array.name()) {
            output.set_active_higher_order_degrees(array.name());
        }
    }
    if let Some(array) = input.process_ids() {
        if output.has_array(array.name()) {
            output.set_active_process_ids(array.name());
        }
    }
}

/// Merge two PolyData meshes.
pub fn merge_two(a: &PolyData, b: &PolyData) -> PolyData {
    merge(&[a, b])
}

/// Apply a translation to all points in a PolyData (returns a copy).
pub fn translate(pd: &PolyData, dx: f64, dy: f64, dz: f64) -> PolyData {
    let mut result = pd.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result.points.set(i, [p[0] + dx, p[1] + dy, p[2] + dz]);
    }
    result
}

/// Apply uniform scaling to all points (returns a copy).
pub fn scale_uniform(pd: &PolyData, factor: f64) -> PolyData {
    let mut result = pd.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [p[0] * factor, p[1] * factor, p[2] * factor]);
    }
    result
}

/// Apply position + scale to create a transformed copy.
pub fn transform_position_scale(pd: &PolyData, position: [f64; 3], scale: f64) -> PolyData {
    let mut result = pd.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result.points.set(
            i,
            [
                p[0] * scale + position[0],
                p[1] * scale + position[1],
                p[2] * scale + position[2],
            ],
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> PolyData {
        PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn merge_two_meshes() {
        let a = tri();
        let b = PolyData::from_triangles(
            vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let merged = merge_two(&a, &b);
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);
        // Second triangle should have offset indices
        assert_eq!(merged.polys.cell(1), &[3, 4, 5]);
    }

    #[test]
    fn merge_many() {
        let a = tri();
        let b = tri();
        let c = tri();
        let merged = merge(&[&a, &b, &c]);
        assert_eq!(merged.points.len(), 9);
        assert_eq!(merged.polys.num_cells(), 3);
    }

    #[test]
    fn merge_empty() {
        let result = merge(&[]);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn translate_mesh() {
        let pd = tri();
        let moved = translate(&pd, 10.0, 0.0, 0.0);
        let p = moved.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn scale_mesh() {
        let pd = tri();
        let scaled = scale_uniform(&pd, 3.0);
        let p = scaled.points.get(1);
        assert!((p[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn transform_pos_scale() {
        let pd = tri();
        let t = transform_position_scale(&pd, [10.0, 0.0, 0.0], 2.0);
        let p = t.points.get(1);
        assert!((p[0] - 12.0).abs() < 1e-10); // 1.0*2 + 10.0
    }
}
