use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};
use std::collections::HashSet;

/// Remove duplicate polygons (same vertices in any order) and degenerate polygons.
pub fn remove_duplicate_cells(input: &PolyData) -> PolyData {
    let mut seen: HashSet<Vec<i64>> = HashSet::new();
    let mut out_polys = CellArray::new();
    let mut kept_polys = Vec::new();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if !cell_ids_are_valid(cell, input.points.len()) {
            continue;
        }
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

/// Remove zero-area (degenerate) polygons.
pub fn remove_degenerate_cells(input: &PolyData, min_area: f64) -> PolyData {
    let min_a2 = min_area * min_area * 4.0; // compare with (2*area)^2 to avoid sqrt
    let mut out_polys = CellArray::new();
    let mut kept_polys = Vec::new();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if !cell_ids_are_valid(cell, input.points.len()) {
            continue;
        }
        let [cx, cy, cz] = polygon_area_vector_x2(input, cell);
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
///
/// Single implementation in [`crate::filters::mesh::remove_unused_points`].
pub use crate::filters::mesh::remove_unused_points::remove_unused_points;

fn cell_ids_are_valid(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn polygon_area_vector_x2(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    let p0 = input.points.get(cell[0] as usize);
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;

    for i in 1..cell.len() - 1 {
        let p1 = input.points.get(cell[i] as usize);
        let p2 = input.points.get(cell[i + 1] as usize);
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        cx += e1[1] * e2[2] - e1[2] * e2[1];
        cy += e1[2] * e2[0] - e1[0] * e2[2];
        cz += e1[0] * e2[1] - e1[1] * e2[0];
    }

    [cx, cy, cz]
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
    fn remove_degenerate_uses_full_polygon_area() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = remove_degenerate_cells(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(remove_duplicate_cells(&pd).polys.num_cells(), 0);
    }
}
