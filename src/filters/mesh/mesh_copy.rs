//! Mesh copy, merge, and append operations.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Deep copy a mesh (no shared references).
pub fn deep_copy_mesh(mesh: &PolyData) -> PolyData {
    mesh.clone()
}

/// Append multiple meshes into one.
///
/// The single implementation lives in [`crate::filters::mesh::merge_ops`].
pub use crate::filters::mesh::merge_ops::append_meshes;

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
