//! Mesh merge operations: append, zip, interleave.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Append multiple meshes into one (no point merging).
pub fn append_meshes(meshes: &[&PolyData]) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    if meshes.len() == 1 {
        return meshes[0].clone();
    }
    let datasets: Vec<&PolyData> = meshes
        .iter()
        .copied()
        .filter(|mesh| !mesh.points.is_empty())
        .collect();
    if datasets.is_empty() {
        return PolyData::new();
    }
    if datasets.len() == 1 {
        return datasets[0].clone();
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    for &m in &datasets {
        let base = pts.len() as i64;
        for i in 0..m.points.len() {
            pts.push(m.points.get(i));
        }
        copy_cells(&m.verts, &mut verts, base);
        copy_cells(&m.lines, &mut lines, base);
        copy_cells(&m.polys, &mut polys, base);
        copy_cells(&m.strips, &mut strips, base);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r.lines = lines;
    r.polys = polys;
    r.strips = strips;
    copy_common_point_data(&datasets, &mut r);
    copy_common_cell_data(&datasets, &mut r);
    if let Some(first) = datasets.first() {
        *r.field_data_mut() = first.field_data().clone();
    }
    r
}

/// Zip two meshes: interleave their triangles.
pub fn zip_meshes(a: &PolyData, b: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Copy A's points
    for i in 0..a.points.len() {
        pts.push(a.points.get(i));
    }
    let offset_b = a.points.len() as i64;
    for i in 0..b.points.len() {
        pts.push(b.points.get(i));
    }

    let a_cells: Vec<Vec<i64>> = a.polys.iter().map(|c| c.to_vec()).collect();
    let b_cells: Vec<Vec<i64>> = b
        .polys
        .iter()
        .map(|c| c.iter().map(|&id| id + offset_b).collect())
        .collect();

    let max_len = a_cells.len().max(b_cells.len());
    for i in 0..max_len {
        if i < a_cells.len() {
            polys.push_cell(&a_cells[i]);
        }
        if i < b_cells.len() {
            polys.push_cell(&b_cells[i]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Stack meshes vertically (translate each by cumulative height along Y).
pub fn stack_vertical(meshes: &[&PolyData], gap: f64) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    let mut y_offset = 0.0;

    for mesh in meshes {
        let base = pts.len() as i64;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for i in 0..mesh.points.len() {
            let p = mesh.points.get(i);
            min_y = min_y.min(p[1]);
            max_y = max_y.max(p[1]);
        }
        let translate_y = if mesh.points.is_empty() {
            y_offset
        } else {
            y_offset - min_y
        };
        for i in 0..mesh.points.len() {
            let p = mesh.points.get(i);
            pts.push([p[0], p[1] + translate_y, p[2]]);
        }
        copy_cells(&mesh.verts, &mut verts, base);
        copy_cells(&mesh.lines, &mut lines, base);
        copy_cells(&mesh.polys, &mut polys, base);
        copy_cells(&mesh.strips, &mut strips, base);
        if !mesh.points.is_empty() {
            y_offset += (max_y - min_y).max(0.0) + gap;
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    result
}

/// Tile a mesh in a grid pattern.
pub fn tile_mesh(mesh: &PolyData, nx: usize, ny: usize, spacing: [f64; 2]) -> PolyData {
    let mut all = Vec::new();
    for iy in 0..ny {
        for ix in 0..nx {
            let mut copy = mesh.clone();
            let mut pts = Points::<f64>::new();
            for i in 0..copy.points.len() {
                let p = copy.points.get(i);
                pts.push([
                    p[0] + ix as f64 * spacing[0],
                    p[1] + iy as f64 * spacing[1],
                    p[2],
                ]);
            }
            copy.points = pts;
            all.push(copy);
        }
    }
    let refs: Vec<&PolyData> = all.iter().collect();
    append_meshes(&refs)
}

fn copy_cells(src: &CellArray, dst: &mut CellArray, offset: i64) {
    for cell in src.iter() {
        let ids: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
        dst.push_cell(&ids);
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
    fn append() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = append_meshes(&[&a, &b]);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
    }
    #[test]
    fn append_ignores_empty_inputs_for_common_arrays() {
        let mut a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let empty = PolyData::new();

        let result = append_meshes(&[&empty, &a]);

        let arr = result.point_data().get_array("temperature").unwrap();
        assert_eq!(arr.to_f64_vec(), vec![1.0, 2.0, 3.0]);
    }
    #[test]
    fn append_cell_data_matches_type_major_output_cell_order() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([0.0, 1.0, 0.0]);
        a.points.push([1.0, 1.0, 0.0]);
        a.verts.push_cell(&[0]);
        a.lines.push_cell(&[0, 1]);
        a.polys.push_cell(&[0, 1, 2]);
        a.strips.push_cell(&[0, 1, 2, 3]);
        a.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_id",
                vec![10.0, 20.0, 30.0, 40.0],
                1,
            )));

        let mut b = a.clone();
        b.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_id",
                vec![11.0, 21.0, 31.0, 41.0],
                1,
            )));

        let result = append_meshes(&[&a, &b]);

        let arr = result.cell_data().get_array("cell_id").unwrap();
        assert_eq!(
            arr.to_f64_vec(),
            vec![10.0, 11.0, 20.0, 21.0, 30.0, 31.0, 40.0, 41.0]
        );
    }
    #[test]
    fn zip() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = zip_meshes(&a, &b);
        assert_eq!(result.polys.num_cells(), 2);
    }
    #[test]
    fn tile() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = tile_mesh(&mesh, 3, 2, [2.0, 2.0]);
        assert_eq!(result.polys.num_cells(), 6);
    }
    #[test]
    fn stack() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = stack_vertical(&[&a, &b], 0.5);
        assert_eq!(result.polys.num_cells(), 2);
    }
}
