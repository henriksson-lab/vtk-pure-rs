use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Extract all unique edges from a PolyData as line segments.
///
/// Each polygon edge appears exactly once in the output, regardless of
/// how many cells share it.
pub fn extract_edges(input: &PolyData) -> PolyData {
    let mut seen: HashMap<(i64, i64), usize> = HashMap::new();
    let mut out_lines = CellArray::new();
    let mut edge_source_cell_ids = Vec::new();

    let insert_edge = |a: i64,
                       b: i64,
                       source_cell_id: usize,
                       seen: &mut HashMap<(i64, i64), usize>,
                       lines: &mut CellArray,
                       edge_source_cell_ids: &mut Vec<usize>| {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&edge_id) = seen.get(&key) {
            edge_source_cell_ids[edge_id] = edge_source_cell_ids[edge_id].min(source_cell_id);
        } else {
            let edge_id = edge_source_cell_ids.len();
            seen.insert(key, edge_id);
            edge_source_cell_ids.push(source_cell_id);
            lines.push_cell(&[a, b]);
        }
    };

    let process_polygon = |cell: &[i64],
                           source_cell_id: usize,
                           seen: &mut HashMap<(i64, i64), usize>,
                           lines: &mut CellArray,
                           edge_source_cell_ids: &mut Vec<usize>| {
        let n = cell.len();
        if n < 2 {
            return;
        }
        for i in 0..n {
            let a = cell[i];
            let b = cell[(i + 1) % n];
            insert_edge(a, b, source_cell_id, seen, lines, edge_source_cell_ids);
        }
    };

    let verts_offset = 0;
    let lines_offset = verts_offset + input.verts.num_cells();
    let polys_offset = lines_offset + input.lines.num_cells();
    let strips_offset = polys_offset + input.polys.num_cells();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        process_polygon(
            cell,
            polys_offset + poly_id,
            &mut seen,
            &mut out_lines,
            &mut edge_source_cell_ids,
        );
    }
    for (strip_id, cell) in input.strips.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let source_cell_id = strips_offset + strip_id;
        for i in 0..cell.len() - 2 {
            let v0 = cell[i];
            let v1 = cell[i + 1];
            let v2 = cell[i + 2];
            insert_edge(
                v0,
                v1,
                source_cell_id,
                &mut seen,
                &mut out_lines,
                &mut edge_source_cell_ids,
            );
            insert_edge(
                v1,
                v2,
                source_cell_id,
                &mut seen,
                &mut out_lines,
                &mut edge_source_cell_ids,
            );
            insert_edge(
                v2,
                v0,
                source_cell_id,
                &mut seen,
                &mut out_lines,
                &mut edge_source_cell_ids,
            );
        }
    }
    // Line cells: edges are consecutive pairs
    for (line_id, cell) in input.lines.iter().enumerate() {
        let source_cell_id = lines_offset + line_id;
        for i in 0..cell.len().saturating_sub(1) {
            let a = cell[i];
            let b = cell[i + 1];
            insert_edge(
                a,
                b,
                source_cell_id,
                &mut seen,
                &mut out_lines,
                &mut edge_source_cell_ids,
            );
        }
    }

    build_output_with_used_points(input, out_lines, &edge_source_cell_ids)
}

fn build_output_with_used_points(
    input: &PolyData,
    lines: CellArray,
    edge_source_cell_ids: &[usize],
) -> PolyData {
    let mut used = vec![false; input.points.len()];
    for cell in lines.iter() {
        for &id in cell {
            if id < 0 || id as usize >= used.len() {
                return PolyData::new();
            }
            used[id as usize] = true;
        }
    }

    let mut old_to_new = vec![-1i64; input.points.len()];
    let mut old_point_ids = Vec::new();
    let mut out_points = Points::<f64>::new();
    for (old_id, &is_used) in used.iter().enumerate() {
        if is_used {
            old_to_new[old_id] = out_points.len() as i64;
            old_point_ids.push(old_id);
            out_points.push(input.points.get(old_id));
        }
    }

    let mut out_lines = CellArray::new();
    for cell in lines.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| old_to_new[id as usize]).collect();
        out_lines.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    copy_point_data(input, &old_point_ids, &mut pd);
    copy_cell_data(input, edge_source_cell_ids, &mut pd);
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

fn select_tuples(array: &AnyDataArray, old_point_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &old_id in old_point_ids {
                out.push_tuple($array.tuple(old_id));
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
    fn edges_of_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extract_edges(&pd);
        assert_eq!(result.lines.num_cells(), 3);
    }

    #[test]
    fn shared_edge_appears_once() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let result = extract_edges(&pd);
        // 2 triangles share edge (0,1): 3 + 3 - 1 = 5 unique edges
        assert_eq!(result.lines.num_cells(), 5);
    }

    #[test]
    fn edges_of_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        let result = extract_edges(&pd);
        assert_eq!(result.lines.num_cells(), 4);
    }

    #[test]
    fn drops_unused_points_like_vtk_default() {
        let mut pd = PolyData::new();
        pd.points.push([99.0, 99.0, 99.0]);
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.point_data_mut()
            .add_array(crate::data::AnyDataArray::F64(
                crate::data::DataArray::from_vec("ids", vec![99.0, 1.0, 2.0, 3.0], 1),
            ));

        let result = extract_edges(&pd);

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.lines.cell(0), &[0, 1]);
        let ids = result.point_data().get_array("ids").unwrap();
        let mut buf = [0.0];
        ids.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn copies_cell_data_from_source_cells() {
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        pd.cell_data_mut().add_array(crate::data::AnyDataArray::I32(
            crate::data::DataArray::from_vec("cell_ids", vec![10, 20], 1),
        ));

        let result = extract_edges(&pd);

        let cell_ids = result.cell_data().get_array("cell_ids").unwrap();
        assert_eq!(cell_ids.num_tuples(), result.lines.num_cells());
        assert_eq!(cell_ids.scalar_type(), crate::types::ScalarType::I32);
    }
}
