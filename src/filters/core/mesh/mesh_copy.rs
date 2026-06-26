//! Mesh copy, merge, and append operations.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Deep copy a mesh (no shared references).
pub fn deep_copy_mesh(mesh: &PolyData) -> PolyData {
    mesh.clone()
}

/// Append multiple meshes into one.
pub fn append_meshes(meshes: &[&PolyData]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();

    for mesh in meshes {
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
    copy_common_point_data(meshes, &mut result);
    copy_common_cell_data(meshes, &mut result);
    if let Some(first) = meshes.first() {
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
            if let Some(appended) = append_array(name, array, inputs, |mesh| mesh.cell_data()) {
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
            output.cell_data_mut().add_array(repeat_array(array, n));
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
    fn test_append_all_cell_arrays() {
        let mesh = mixed_cell_mesh();
        let r = append_meshes(&[&mesh, &mesh]);
        assert_eq!(r.verts.num_cells(), 2);
        assert_eq!(r.lines.cell(1), &[4, 5]);
        assert_eq!(r.polys.cell(1), &[4, 5, 6]);
        assert_eq!(r.strips.cell(1), &[4, 5, 6, 7]);
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
    fn test_duplicate_all_cell_arrays() {
        let mesh = mixed_cell_mesh();
        let r = duplicate_mesh(&mesh, 2, [10.0, 0.0, 0.0]);
        assert_eq!(r.verts.num_cells(), 2);
        assert_eq!(r.lines.cell(1), &[4, 5]);
        assert_eq!(r.polys.cell(1), &[4, 5, 6]);
        assert_eq!(r.strips.cell(1), &[4, 5, 6, 7]);
    }

    #[test]
    fn append_preserves_common_point_and_cell_data() {
        let mut a = mixed_cell_mesh();
        let mut b = mixed_cell_mesh();
        add_mesh_arrays(&mut a, 0.0);
        add_mesh_arrays(&mut b, 10.0);

        let r = append_meshes(&[&a, &b]);
        let point_ids = r.point_data().get_array("pid").unwrap();
        let cell_ids = r.cell_data().get_array("cid").unwrap();
        assert_eq!(point_ids.num_tuples(), 8);
        assert_eq!(cell_ids.num_tuples(), 8);

        let mut buf = [0.0];
        point_ids.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 10.0);
        cell_ids.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 10.0);
    }

    #[test]
    fn duplicate_repeats_point_and_cell_data() {
        let mut mesh = mixed_cell_mesh();
        add_mesh_arrays(&mut mesh, 3.0);

        let r = duplicate_mesh(&mesh, 2, [10.0, 0.0, 0.0]);
        let point_ids = r.point_data().get_array("pid").unwrap();
        let cell_ids = r.cell_data().get_array("cid").unwrap();
        assert_eq!(point_ids.num_tuples(), 8);
        assert_eq!(cell_ids.num_tuples(), 8);

        let mut buf = [0.0];
        point_ids.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 3.0);
        cell_ids.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 3.0);
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

    fn add_mesh_arrays(mesh: &mut PolyData, base: f64) {
        let num_points = mesh.points.len();
        let num_cells = mesh.total_cells();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "pid",
                (0..num_points).map(|i| base + i as f64).collect(),
                1,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cid",
                (0..num_cells).map(|i| base + i as f64).collect(),
                1,
            )));
    }
}
