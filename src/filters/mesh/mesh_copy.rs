//! Mesh copy, merge, and append operations.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Deep copy a mesh (no shared references).
pub fn deep_copy_mesh(mesh: &PolyData) -> PolyData {
    mesh.clone()
}

/// Append multiple meshes into one.
pub fn append_meshes(meshes: &[&PolyData]) -> PolyData {
    let datasets: Vec<&PolyData> = meshes
        .iter()
        .copied()
        .filter(|mesh| !mesh.points.is_empty())
        .collect();
    if datasets.is_empty() {
        return PolyData::new();
    }

    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();

    for mesh in &datasets {
        let offset = pts.len() as i64;
        for i in 0..mesh.points.len() {
            pts.push(mesh.points.get(i));
        }
        copy_cells(&mesh.verts, &mut verts, offset);
        copy_cells(&mesh.lines, &mut lines, offset);
        copy_cells(&mesh.polys, &mut polys, offset);
        copy_cells(&mesh.strips, &mut strips, offset);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    copy_common_point_data(&datasets, &mut result);
    copy_common_cell_data(&datasets, &mut result);
    if let Some(first) = datasets.first() {
        *result.field_data_mut() = first.field_data().clone();
    }
    result
}

/// Duplicate mesh N times with given offset between copies.
pub fn duplicate_mesh(mesh: &PolyData, n: usize, offset: [f64; 3]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();

    for copy in 0..n {
        let pt_offset = pts.len() as i64;
        let dx = offset[0] * copy as f64;
        let dy = offset[1] * copy as f64;
        let dz = offset[2] * copy as f64;
        for i in 0..mesh.points.len() {
            let p = mesh.points.get(i);
            pts.push([p[0] + dx, p[1] + dy, p[2] + dz]);
        }
        copy_cells(&mesh.verts, &mut verts, pt_offset);
        copy_cells(&mesh.lines, &mut lines, pt_offset);
        copy_cells(&mesh.polys, &mut polys, pt_offset);
        copy_cells(&mesh.strips, &mut strips, pt_offset);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    repeat_point_data(mesh, n, &mut result);
    repeat_cell_data(mesh, n, &mut result);
    *result.field_data_mut() = mesh.field_data().clone();
    result
}

fn copy_cells(src: &CellArray, dst: &mut CellArray, offset: i64) {
    for cell in src.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
        dst.push_cell(&shifted);
    }
}

fn copy_common_point_data(inputs: &[&PolyData], output: &mut PolyData) {
    let Some(first) = inputs.first() else {
        return;
    };
    for array in first.point_data().field_data().iter() {
        let name = array.name();
        if array.num_tuples() != first.points.len() {
            continue;
        }
        if inputs.iter().all(|mesh| {
            mesh.point_data().get_array(name).is_some_and(|other| {
                arrays_compatible(array, other) && other.num_tuples() == mesh.points.len()
            })
        }) {
            if let Some(appended) = append_array(name, array, inputs, |mesh| mesh.point_data()) {
                output.point_data_mut().add_array(appended);
            }
        }
    }
    copy_active_attributes(first.point_data(), output.point_data_mut());
}

fn copy_common_cell_data(inputs: &[&PolyData], output: &mut PolyData) {
    let Some(first) = inputs.first() else {
        return;
    };
    for array in first.cell_data().field_data().iter() {
        let name = array.name();
        if array.num_tuples() != first.total_cells() {
            continue;
        }
        if inputs.iter().all(|mesh| {
            mesh.cell_data().get_array(name).is_some_and(|other| {
                arrays_compatible(array, other) && other.num_tuples() == mesh.total_cells()
            })
        }) {
            if let Some(appended) = append_cell_array(name, array, inputs) {
                output.cell_data_mut().add_array(appended);
            }
        }
    }
    copy_active_attributes(first.cell_data(), output.cell_data_mut());
}

fn repeat_point_data(input: &PolyData, n: usize, output: &mut PolyData) {
    for array in input.point_data().field_data().iter() {
        if array.num_tuples() == input.points.len() {
            output.point_data_mut().add_array(repeat_array(array, n));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn repeat_cell_data(input: &PolyData, n: usize, output: &mut PolyData) {
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(repeat_cell_array(array, input, n));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn append_array(
    name: &str,
    template: &AnyDataArray,
    inputs: &[&PolyData],
    attrs: impl Fn(&PolyData) -> &DataSetAttributes,
) -> Option<AnyDataArray> {
    macro_rules! append {
        ($variant:ident, $ty:ty) => {{
            let mut out = DataArray::<$ty>::new(name, template.num_components());
            for mesh in inputs {
                let Some(AnyDataArray::$variant(array)) = attrs(mesh).get_array(name) else {
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
            for range_fn in [
                cell_vert_range as fn(&PolyData) -> std::ops::Range<usize>,
                cell_line_range,
                cell_poly_range,
                cell_strip_range,
            ] {
                for mesh in inputs {
                    let Some(AnyDataArray::$variant(array)) = mesh.cell_data().get_array(name)
                    else {
                        return None;
                    };
                    for tuple_idx in range_fn(mesh) {
                        out.push_tuple(array.tuple(tuple_idx));
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

fn repeat_array(array: &AnyDataArray, n: usize) -> AnyDataArray {
    macro_rules! repeat {
        ($variant:ident) => {{
            let AnyDataArray::$variant(data_array) = array else {
                unreachable!();
            };
            let mut data = Vec::with_capacity(data_array.as_slice().len() * n);
            for _ in 0..n {
                data.extend_from_slice(data_array.as_slice());
            }
            AnyDataArray::$variant(DataArray::from_vec(
                data_array.name(),
                data,
                data_array.num_components(),
            ))
        }};
    }

    match array {
        AnyDataArray::F32(_) => repeat!(F32),
        AnyDataArray::F64(_) => repeat!(F64),
        AnyDataArray::I8(_) => repeat!(I8),
        AnyDataArray::I16(_) => repeat!(I16),
        AnyDataArray::I32(_) => repeat!(I32),
        AnyDataArray::I64(_) => repeat!(I64),
        AnyDataArray::U8(_) => repeat!(U8),
        AnyDataArray::U16(_) => repeat!(U16),
        AnyDataArray::U32(_) => repeat!(U32),
        AnyDataArray::U64(_) => repeat!(U64),
    }
}

fn repeat_cell_array(array: &AnyDataArray, input: &PolyData, n: usize) -> AnyDataArray {
    macro_rules! repeat {
        ($variant:ident) => {{
            let AnyDataArray::$variant(data_array) = array else {
                unreachable!();
            };
            let mut out = DataArray::new(data_array.name(), data_array.num_components());
            for range_fn in [
                cell_vert_range as fn(&PolyData) -> std::ops::Range<usize>,
                cell_line_range,
                cell_poly_range,
                cell_strip_range,
            ] {
                let range = range_fn(input);
                for _ in 0..n {
                    for tuple_idx in range.clone() {
                        out.push_tuple(data_array.tuple(tuple_idx));
                    }
                }
            }
            AnyDataArray::$variant(out)
        }};
    }

    match array {
        AnyDataArray::F32(_) => repeat!(F32),
        AnyDataArray::F64(_) => repeat!(F64),
        AnyDataArray::I8(_) => repeat!(I8),
        AnyDataArray::I16(_) => repeat!(I16),
        AnyDataArray::I32(_) => repeat!(I32),
        AnyDataArray::I64(_) => repeat!(I64),
        AnyDataArray::U8(_) => repeat!(U8),
        AnyDataArray::U16(_) => repeat!(U16),
        AnyDataArray::U32(_) => repeat!(U32),
        AnyDataArray::U64(_) => repeat!(U64),
    }
}

fn cell_vert_range(mesh: &PolyData) -> std::ops::Range<usize> {
    0..mesh.verts.num_cells()
}

fn cell_line_range(mesh: &PolyData) -> std::ops::Range<usize> {
    let start = mesh.verts.num_cells();
    start..start + mesh.lines.num_cells()
}

fn cell_poly_range(mesh: &PolyData) -> std::ops::Range<usize> {
    let start = mesh.verts.num_cells() + mesh.lines.num_cells();
    start..start + mesh.polys.num_cells()
}

fn cell_strip_range(mesh: &PolyData) -> std::ops::Range<usize> {
    let start = mesh.verts.num_cells() + mesh.lines.num_cells() + mesh.polys.num_cells();
    start..start + mesh.strips.num_cells()
}

fn arrays_compatible(a: &AnyDataArray, b: &AnyDataArray) -> bool {
    a.scalar_type() == b.scalar_type() && a.num_components() == b.num_components()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_append() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.5, 6.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = append_meshes(&[&a, &b]);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_duplicate() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = duplicate_mesh(&mesh, 3, [2.0, 0.0, 0.0]);
        assert_eq!(r.points.len(), 9);
        assert_eq!(r.polys.num_cells(), 3);
        let p = r.points.get(3);
        assert!((p[0] - 2.0).abs() < 1e-10); // second copy offset by 2
    }

    #[test]
    fn append_cell_data_matches_output_cell_order() {
        let mut a = mixed_cell_mesh();
        let mut b = mixed_cell_mesh();
        set_cell_ids(&mut a, [10.0, 20.0, 30.0, 40.0]);
        set_cell_ids(&mut b, [11.0, 21.0, 31.0, 41.0]);

        let r = append_meshes(&[&a, &b]);

        assert_eq!(
            r.cell_data().get_array("cell_id").unwrap().to_f64_vec(),
            vec![10.0, 11.0, 20.0, 21.0, 30.0, 31.0, 40.0, 41.0]
        );
    }

    #[test]
    fn append_ignores_empty_inputs_for_common_arrays() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let empty = PolyData::new();

        let r = append_meshes(&[&empty, &mesh]);

        assert_eq!(r.points.len(), 3);
        assert_eq!(
            r.point_data()
                .get_array("temperature")
                .unwrap()
                .to_f64_vec(),
            vec![1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn duplicate_cell_data_matches_output_cell_order() {
        let mut mesh = mixed_cell_mesh();
        set_cell_ids(&mut mesh, [10.0, 20.0, 30.0, 40.0]);

        let r = duplicate_mesh(&mesh, 2, [10.0, 0.0, 0.0]);

        assert_eq!(
            r.cell_data().get_array("cell_id").unwrap().to_f64_vec(),
            vec![10.0, 10.0, 20.0, 20.0, 30.0, 30.0, 40.0, 40.0]
        );
    }

    fn mixed_cell_mesh() -> PolyData {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);
        mesh
    }

    fn set_cell_ids(mesh: &mut PolyData, values: [f64; 4]) {
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_id",
                values.to_vec(),
                1,
            )));
    }
}
