//! StaticCleanPolyData - merge duplicate points and remove unused points using spatial hashing.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use std::collections::HashMap;

fn remap_and_filter_cells(
    ca: &CellArray,
    point_map: &[Option<usize>],
    minimum_size: usize,
    cell_offset: usize,
    kept_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for (cell_id, cell) in ca.iter().enumerate() {
        let Some(mapped) = remap_unique_cell(cell, point_map) else {
            continue;
        };
        if mapped.len() >= minimum_size {
            out.push_cell(&mapped);
            kept_cell_ids.push(cell_offset + cell_id);
        }
    }
    out
}

/// Merge duplicate points, remove unused points, and drop cells whose merged
/// connectivity is degenerate under vtkStaticCleanPolyData's default flags.
///
/// `tolerance` controls the absolute merge distance for duplicate points.
pub fn static_clean_poly_data(input: &PolyData, tolerance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let tolerance = tolerance.max(0.0);
    let tolerance2 = tolerance * tolerance;

    let inv_cell = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1.0e10
    };
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    let mut merge_map = vec![0usize; n];
    let mut representatives = Vec::new();
    let mut representative_points = Points::<f64>::new();

    for i in 0..n {
        let p = input.points.get(i);
        let gx = (p[0] * inv_cell).floor() as i64;
        let gy = (p[1] * inv_cell).floor() as i64;
        let gz = (p[2] * inv_cell).floor() as i64;

        let mut found = None;

        'outer: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(bucket) = grid.get(&(gx + dx, gy + dy, gz + dz)) {
                        for &rep_id in bucket {
                            let q = representative_points.get(rep_id);
                            let d2 = (p[0] - q[0]).powi(2)
                                + (p[1] - q[1]).powi(2)
                                + (p[2] - q[2]).powi(2);
                            if d2 <= tolerance2 {
                                found = Some(rep_id);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if let Some(rep_id) = found {
            merge_map[i] = rep_id;
        } else {
            let rep_id = representatives.len();
            representatives.push(i);
            representative_points.push(p);
            merge_map[i] = rep_id;
            grid.entry((gx, gy, gz)).or_default().push(rep_id);
        }
    }

    let mut used_representatives = vec![false; representatives.len()];
    mark_used_representatives(&input.verts, &merge_map, &mut used_representatives);
    mark_used_representatives(&input.lines, &merge_map, &mut used_representatives);
    mark_used_representatives(&input.polys, &merge_map, &mut used_representatives);
    mark_used_representatives(&input.strips, &merge_map, &mut used_representatives);

    let mut point_map = vec![None; n];
    let mut new_pts = Points::<f64>::new();
    let mut output_representatives = Vec::new();
    let mut rep_to_output = vec![None; representatives.len()];
    for (rep_id, &old_id) in representatives.iter().enumerate() {
        if used_representatives[rep_id] {
            let new_id = new_pts.len();
            new_pts.push(input.points.get(old_id));
            output_representatives.push(old_id);
            rep_to_output[rep_id] = Some(new_id);
        }
    }
    for old_id in 0..n {
        point_map[old_id] = rep_to_output[merge_map[old_id]];
    }

    let mut kept_cell_ids = Vec::new();
    let verts = remap_and_filter_cells(&input.verts, &point_map, 1, 0, &mut kept_cell_ids);
    let line_offset = input.verts.num_cells();
    let lines =
        remap_and_filter_cells(&input.lines, &point_map, 2, line_offset, &mut kept_cell_ids);
    let poly_offset = line_offset + input.lines.num_cells();
    let polys =
        remap_and_filter_cells(&input.polys, &point_map, 3, poly_offset, &mut kept_cell_ids);
    let strip_offset = poly_offset + input.polys.num_cells();
    let strips = remap_and_filter_cells(
        &input.strips,
        &point_map,
        4,
        strip_offset,
        &mut kept_cell_ids,
    );

    let mut result = input.clone();
    result.points = new_pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    remap_point_data(input, &output_representatives, &mut result);
    remap_cell_data(input, &kept_cell_ids, &mut result);
    result
}

fn mark_used_representatives(cells: &CellArray, merge_map: &[usize], used: &mut [bool]) {
    for cell in cells.iter() {
        for &id in cell {
            let Ok(old_id) = usize::try_from(id) else {
                continue;
            };
            if old_id < merge_map.len() {
                used[merge_map[old_id]] = true;
            }
        }
    }
}

fn remap_unique_cell(cell: &[i64], point_map: &[Option<usize>]) -> Option<Vec<i64>> {
    let mut mapped = Vec::new();
    for &id in cell {
        let old_id = usize::try_from(id).ok()?;
        let Some(Some(new_id)) = point_map.get(old_id) else {
            return None;
        };
        let new_id = *new_id;
        if !mapped.contains(&(new_id as i64)) {
            mapped.push(new_id as i64);
        }
    }
    Some(mapped)
}

fn remap_point_data(input: &PolyData, representatives: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, representatives));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn remap_cell_data(input: &PolyData, kept_cell_ids: &[usize], output: &mut PolyData) {
    output.cell_data_mut().clear();
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, kept_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_tuples(array: &AnyDataArray, representatives: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in representatives {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_polys_with_fewer_than_three_unique_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([5.0, 5.0, 5.0]);
        pd.points.push([5.0, 5.0, 5.0]);
        pd.points.push([5.0, 5.0, 5.0]);

        pd.polys.push_cell(&[0, 1, 2]); // good triangle
        pd.polys.push_cell(&[3, 4, 5]); // degenerate

        let result = static_clean_poly_data(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn merges_duplicate_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.001]); // near-duplicate of point 0

        pd.polys.push_cell(&[0, 1, 2]);

        let result = static_clean_poly_data(&pd, 0.01);
        // Point 3 should be merged with point 0
        assert!(result.points.len() <= 3);
    }

    #[test]
    fn preserves_valid_mesh() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = static_clean_poly_data(&pd, 1e-8);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn remaps_cell_data_to_surviving_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([5.0, 5.0, 5.0]);
        pd.points.push([5.0, 5.0, 5.0]);
        pd.points.push([5.0, 5.0, 5.0]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![10, 20],
                1,
            )));

        let result = static_clean_poly_data(&pd, 0.01);
        let array = result.cell_data().get_array("cell_id").unwrap();
        assert_eq!(array.num_tuples(), 1);
        let mut value = [0.0];
        array.tuple_as_f64(0, &mut value);
        assert_eq!(value[0] as i32, 20);
    }
}
