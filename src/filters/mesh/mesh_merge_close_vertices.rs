//! Merge vertices that are closer than a threshold distance.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn merge_close_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || !tolerance.is_finite() || tolerance < 0.0 {
        return mesh.clone();
    }
    let tol2 = tolerance * tolerance;
    let mut map = vec![0usize; n];
    let mut old_point_ids = Vec::new();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let mut merged = false;
        for j in 0..pts.len() {
            let q = pts.get(j);
            if (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) <= tol2 {
                map[i] = j;
                merged = true;
                break;
            }
        }
        if !merged {
            map[i] = pts.len();
            old_point_ids.push(i);
            pts.push(p.try_into().unwrap());
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    let mut kept_cell_ids = Vec::new();
    let mut cell_offset = 0usize;
    result.verts = remap_cell_array(&mesh.verts, &map, n, 1, cell_offset, &mut kept_cell_ids);
    cell_offset += mesh.verts.num_cells();
    result.lines = remap_cell_array(&mesh.lines, &map, n, 2, cell_offset, &mut kept_cell_ids);
    cell_offset += mesh.lines.num_cells();
    result.polys = remap_cell_array(&mesh.polys, &map, n, 3, cell_offset, &mut kept_cell_ids);
    cell_offset += mesh.polys.num_cells();
    result.strips = remap_cell_array(&mesh.strips, &map, n, 3, cell_offset, &mut kept_cell_ids);
    copy_point_data(
        mesh.point_data(),
        result.point_data_mut(),
        &old_point_ids,
        n,
    );
    copy_cell_data(
        mesh.cell_data(),
        result.cell_data_mut(),
        &kept_cell_ids,
        mesh.total_cells(),
    );
    *result.field_data_mut() = mesh.field_data().clone();
    result
}

fn remap_cell_array(
    cells: &CellArray,
    point_map: &[usize],
    num_input_points: usize,
    min_unique: usize,
    old_cell_offset: usize,
    old_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let mapped = cell
            .iter()
            .map(|&point_id| {
                usize::try_from(point_id)
                    .ok()
                    .filter(|&idx| idx < num_input_points)
                    .map(|idx| point_map[idx] as i64)
            })
            .collect::<Option<Vec<i64>>>();
        let Some(mapped) = mapped else {
            continue;
        };
        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= min_unique {
            out.push_cell(&mapped);
            old_cell_ids.push(old_cell_offset + cell_id);
        }
    }
    out
}

fn copy_point_data(
    input: &DataSetAttributes,
    output: &mut DataSetAttributes,
    old_point_ids: &[usize],
    num_input_points: usize,
) {
    output.clear();
    for array in input.field_data().iter() {
        if array.num_tuples() == num_input_points {
            output.add_array(remap_array(array, old_point_ids));
        }
    }
    copy_active_attributes(input, output);
}

fn copy_cell_data(
    input: &DataSetAttributes,
    output: &mut DataSetAttributes,
    old_cell_ids: &[usize],
    num_input_cells: usize,
) {
    output.clear();
    for array in input.field_data().iter() {
        if array.num_tuples() == num_input_cells {
            output.add_array(remap_array(array, old_cell_ids));
        }
    }
    copy_active_attributes(input, output);
}

fn remap_array(array: &AnyDataArray, old_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($array, old_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => remap!(array, F32),
        AnyDataArray::F64(array) => remap!(array, F64),
        AnyDataArray::I8(array) => remap!(array, I8),
        AnyDataArray::I16(array) => remap!(array, I16),
        AnyDataArray::I32(array) => remap!(array, I32),
        AnyDataArray::I64(array) => remap!(array, I64),
        AnyDataArray::U8(array) => remap!(array, U8),
        AnyDataArray::U16(array) => remap!(array, U16),
        AnyDataArray::U32(array) => remap!(array, U32),
        AnyDataArray::U64(array) => remap!(array, U64),
    }
}

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_ids: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(old_ids.len() * num_components);
    for &old_id in old_ids {
        data.extend_from_slice(array.tuple(old_id));
    }
    DataArray::from_vec(array.name(), data, num_components)
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
    fn test_merge() {
        let mut mesh = PolyData::new();
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([0.001, 0.0, 0.0]); // very close
        pts.push([1.0, 0.0, 0.0]);
        pts.push([0.5, 1.0, 0.0]);
        mesh.points = pts;
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 2, 3]);
        polys.push_cell(&[1, 2, 3]);
        mesh.polys = polys;
        let r = merge_close_vertices(&mesh, 0.01);
        assert_eq!(r.points.len(), 3); // 0 and 1 merged
    }
}
