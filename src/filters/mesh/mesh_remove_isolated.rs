//! Remove isolated vertices (vertices not referenced by any cell).
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

pub fn remove_isolated_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut used = vec![false; n];
    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        for cell in cells.iter() {
            if !valid_cell_points(cell, n) {
                continue;
            }
            for &v in cell {
                used[v as usize] = true;
            }
        }
    }

    let mut pt_map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    let mut original_ids = Vec::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
            original_ids.push(i);
        }
    }

    let remap = |cells: &CellArray| -> CellArray {
        let mut out = CellArray::new();
        for cell in cells.iter() {
            if !valid_cell_points(cell, n) {
                continue;
            }
            let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
            out.push_cell(&mapped);
        }
        out
    };

    let mut result = mesh.clone();
    result.points = pts;
    result.verts = remap(&mesh.verts);
    result.lines = remap(&mesh.lines);
    result.polys = remap(&mesh.polys);
    result.strips = remap(&mesh.strips);
    replace_point_data(result.point_data_mut(), mesh.point_data(), &original_ids, n);
    result
}

fn valid_cell_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| valid_point_id(id, num_points).is_some())
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
}

fn replace_point_data(
    output: &mut DataSetAttributes,
    input: &DataSetAttributes,
    original_ids: &[usize],
    num_input_points: usize,
) {
    let active_scalars = input.scalars().map(|a| a.name().to_string());
    let active_vectors = input.vectors().map(|a| a.name().to_string());
    let active_normals = input.normals().map(|a| a.name().to_string());
    let active_tcoords = input.tcoords().map(|a| a.name().to_string());
    let active_tensors = input.tensors().map(|a| a.name().to_string());
    let active_global_ids = input.global_ids().map(|a| a.name().to_string());
    let active_pedigree_ids = input.pedigree_ids().map(|a| a.name().to_string());
    let active_edge_flags = input.edge_flags().map(|a| a.name().to_string());
    let active_tangents = input.tangents().map(|a| a.name().to_string());
    let active_rational_weights = input.rational_weights().map(|a| a.name().to_string());
    let active_higher_order_degrees = input.higher_order_degrees().map(|a| a.name().to_string());
    let active_process_ids = input.process_ids().map(|a| a.name().to_string());

    output.clear();
    for array in input.field_data().iter() {
        if array.num_tuples() == num_input_points {
            output.add_array(compact_array(array, original_ids));
        }
    }

    if let Some(name) = active_scalars.as_deref() {
        output.set_active_scalars(name);
    }
    if let Some(name) = active_vectors.as_deref() {
        output.set_active_vectors(name);
    }
    if let Some(name) = active_normals.as_deref() {
        output.set_active_normals(name);
    }
    if let Some(name) = active_tcoords.as_deref() {
        output.set_active_tcoords(name);
    }
    if let Some(name) = active_tensors.as_deref() {
        output.set_active_tensors(name);
    }
    if let Some(name) = active_global_ids.as_deref() {
        output.set_active_global_ids(name);
    }
    if let Some(name) = active_pedigree_ids.as_deref() {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = active_edge_flags.as_deref() {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = active_tangents.as_deref() {
        output.set_active_tangents(name);
    }
    if let Some(name) = active_rational_weights.as_deref() {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = active_higher_order_degrees.as_deref() {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = active_process_ids.as_deref() {
        output.set_active_process_ids(name);
    }
}

fn compact_array(array: &AnyDataArray, original_ids: &[usize]) -> AnyDataArray {
    macro_rules! compact {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut data = Vec::with_capacity(original_ids.len() * num_components);
            for &source_id in original_ids {
                data.extend_from_slice($array.tuple(source_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), data, num_components))
        }};
    }

    match array {
        AnyDataArray::F32(a) => compact!(a, F32),
        AnyDataArray::F64(a) => compact!(a, F64),
        AnyDataArray::I8(a) => compact!(a, I8),
        AnyDataArray::I16(a) => compact!(a, I16),
        AnyDataArray::I32(a) => compact!(a, I32),
        AnyDataArray::I64(a) => compact!(a, I64),
        AnyDataArray::U8(a) => compact!(a, U8),
        AnyDataArray::U16(a) => compact!(a, U16),
        AnyDataArray::U32(a) => compact!(a, U32),
        AnyDataArray::U64(a) => compact!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_isolated() {
        let mut mesh = PolyData::new();
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([1.0, 0.0, 0.0]);
        pts.push([0.5, 1.0, 0.0]);
        pts.push([99.0, 99.0, 99.0]); // isolated
        mesh.points = pts;
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        mesh.polys = polys;
        let r = remove_isolated_vertices(&mesh);
        assert_eq!(r.points.len(), 3);
    }
}
