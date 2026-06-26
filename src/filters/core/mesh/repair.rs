use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use std::collections::HashSet;

/// Remove duplicate polygons (same vertices in any order) and degenerate polygons.
pub fn remove_duplicate_cells(input: &PolyData) -> PolyData {
    let mut seen: HashSet<Vec<i64>> = HashSet::new();
    let mut out_polys = CellArray::new();
    let mut kept_polys = Vec::new();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        let mut key = cell.to_vec();
        key.sort_unstable();
        key.dedup();

        if key.len() != cell.len() {
            continue;
        }

        if seen.insert(key) {
            out_polys.push_cell(cell);
            kept_polys.push(poly_id);
        }
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    remap_cell_data_for_kept_polys(input, &kept_polys, &mut pd);
    pd
}

/// Remove zero-area (degenerate) triangles.
pub fn remove_degenerate_cells(input: &PolyData, min_area: f64) -> PolyData {
    let min_a2 = min_area * min_area * 4.0; // compare with 4*area^2 to avoid sqrt
    let mut out_polys = CellArray::new();
    let mut kept_polys = Vec::new();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if !cell_ids_are_valid(cell, input.points.len()) {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        let area2_x4 = cx * cx + cy * cy + cz * cz;
        if area2_x4 >= min_a2 {
            out_polys.push_cell(cell);
            kept_polys.push(poly_id);
        }
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    remap_cell_data_for_kept_polys(input, &kept_polys, &mut pd);
    pd
}

/// Remove isolated vertices (points not referenced by any cell).
pub fn remove_unused_points(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut used = vec![false; n];

    if !mark_used(&input.polys, &mut used)
        || !mark_used(&input.lines, &mut used)
        || !mark_used(&input.verts, &mut used)
        || !mark_used(&input.strips, &mut used)
    {
        return PolyData::new();
    }

    if used.iter().all(|&is_used| is_used) {
        return input.clone();
    }

    let mut pt_map = vec![-1i64; n];
    let mut out_points = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = out_points.len() as i64;
            out_points.push(input.points.get(i));
        }
    }

    let remap = |cell: &[i64]| -> Vec<i64> { cell.iter().map(|&id| pt_map[id as usize]).collect() };

    let mut out_polys = CellArray::new();
    for cell in input.polys.iter() {
        out_polys.push_cell(&remap(cell));
    }
    let mut out_lines = CellArray::new();
    for cell in input.lines.iter() {
        out_lines.push_cell(&remap(cell));
    }
    let mut out_verts = CellArray::new();
    for cell in input.verts.iter() {
        out_verts.push_cell(&remap(cell));
    }
    let mut out_strips = CellArray::new();
    for cell in input.strips.iter() {
        out_strips.push_cell(&remap(cell));
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd.verts = out_verts;
    pd.strips = out_strips;
    remap_point_data(input, &used, &mut pd);
    pd
}

fn mark_used(cells: &CellArray, used: &mut [bool]) -> bool {
    for cell in cells.iter() {
        for &id in cell {
            if id < 0 || id as usize >= used.len() {
                return false;
            }
            used[id as usize] = true;
        }
    }
    true
}

fn cell_ids_are_valid(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn remap_point_data(input: &PolyData, used: &[bool], output: &mut PolyData) {
    output.point_data_mut().clear();

    for array in input.point_data().iter() {
        if array.num_tuples() == used.len() {
            output
                .point_data_mut()
                .add_array(select_tuples_by_mask(array, used));
        }
    }

    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn remap_cell_data_for_kept_polys(input: &PolyData, kept_polys: &[usize], output: &mut PolyData) {
    let total_cells = input.total_cells();
    let poly_offset = input.verts.num_cells() + input.lines.num_cells();
    let mut kept = Vec::with_capacity(
        input.verts.num_cells()
            + input.lines.num_cells()
            + kept_polys.len()
            + input.strips.num_cells(),
    );

    kept.extend(0..poly_offset);
    kept.extend(kept_polys.iter().map(|&poly_id| poly_offset + poly_id));
    let strip_offset = poly_offset + input.polys.num_cells();
    kept.extend(strip_offset..total_cells);

    output.cell_data_mut().clear();
    for array in input.cell_data().iter() {
        if array.num_tuples() == total_cells {
            output
                .cell_data_mut()
                .add_array(select_tuples_by_indices(array, &kept));
        }
    }

    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_tuples_by_mask(array: &AnyDataArray, used: &[bool]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for (tuple_id, &is_used) in used.iter().enumerate() {
                if is_used {
                    out.push_tuple($array.tuple(tuple_id));
                }
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

fn select_tuples_by_indices(array: &AnyDataArray, kept: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in kept {
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
    fn remove_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 2]); // duplicate
        pd.polys.push_cell(&[2, 1, 0]); // same vertices different order
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.polys.push_cell(&[3, 2, 1, 0]); // duplicate quad
        pd.polys.push_cell(&[0, 1, 1, 3]); // degenerate

        let result = remove_duplicate_cells(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn remove_degenerate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]); // degenerate
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([0.0, 0.001, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // good
        pd.polys.push_cell(&[3, 4, 5]); // tiny

        let result = remove_degenerate_cells(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn remove_unused() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // 0: used
        pd.points.push([1.0, 0.0, 0.0]); // 1: used
        pd.points.push([0.0, 1.0, 0.0]); // 2: used
        pd.points.push([5.0, 5.0, 5.0]); // 3: unused
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(remove_duplicate_cells(&pd).polys.num_cells(), 0);
        assert_eq!(remove_unused_points(&pd).points.len(), 0);
    }

    #[test]
    fn remove_unused_preserves_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([99.0, 99.0, 99.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.strips.num_cells(), 1);
    }

    #[test]
    fn invalid_point_id_returns_empty_output() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, -1]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.total_cells(), 0);
    }
}
