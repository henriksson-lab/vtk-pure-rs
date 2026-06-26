//! Extract a sub-region from a RectilinearGrid.
//!
//! Extracts a sub-grid by index ranges, preserving coordinate arrays
//! and interpolating point/cell data.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, RectilinearGrid};

/// Extract a sub-region of a RectilinearGrid by index ranges.
///
/// The ranges are inclusive: [x_start..=x_end, y_start..=y_end, z_start..=z_end].
/// Point data and cell data are extracted for the sub-region.
pub fn extract_rectilinear_sub_grid(
    grid: &RectilinearGrid,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
) -> RectilinearGrid {
    let dims = grid.dimensions();
    let x_start = x_range.0.min(dims[0].saturating_sub(1));
    let x_end = x_range.1.min(dims[0].saturating_sub(1));
    let y_start = y_range.0.min(dims[1].saturating_sub(1));
    let y_end = y_range.1.min(dims[1].saturating_sub(1));
    let z_start = z_range.0.min(dims[2].saturating_sub(1));
    let z_end = z_range.1.min(dims[2].saturating_sub(1));

    if x_start > x_end || y_start > y_end || z_start > z_end {
        return RectilinearGrid::new();
    }

    let new_x: Vec<f64> = grid.x_coords()[x_start..=x_end].to_vec();
    let new_y: Vec<f64> = grid.y_coords()[y_start..=y_end].to_vec();
    let new_z: Vec<f64> = grid.z_coords()[z_start..=z_end].to_vec();

    let mut result = RectilinearGrid::from_coords(new_x, new_y, new_z);

    // Extract point data
    let new_dims = result.dimensions();
    let pd = grid.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let mut tuple_ids = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2]);

            for iz in z_start..=z_end {
                for iy in y_start..=y_end {
                    for ix in x_start..=x_end {
                        let old_idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                        tuple_ids.push(old_idx);
                    }
                }
            }

            result
                .point_data_mut()
                .add_array(select_tuples(arr, &tuple_ids));
        }
    }
    copy_active_attributes(grid.point_data(), result.point_data_mut());

    // Extract cell data
    let cd = grid.cell_data();
    let cdims = [
        dims[0].saturating_sub(1).max(1),
        dims[1].saturating_sub(1).max(1),
        dims[2].saturating_sub(1).max(1),
    ];
    let new_cdims = [
        new_dims[0].saturating_sub(1).max(1),
        new_dims[1].saturating_sub(1).max(1),
        new_dims[2].saturating_sub(1).max(1),
    ];

    for ai in 0..cd.num_arrays() {
        if let Some(arr) = cd.get_array_by_index(ai) {
            let mut tuple_ids = Vec::with_capacity(new_cdims[0] * new_cdims[1] * new_cdims[2]);

            for iz in z_start..z_start + new_cdims[2] {
                for iy in y_start..y_start + new_cdims[1] {
                    for ix in x_start..x_start + new_cdims[0] {
                        let old_idx = ix + iy * cdims[0] + iz * cdims[0] * cdims[1];
                        if old_idx < arr.num_tuples() {
                            tuple_ids.push(old_idx);
                        }
                    }
                }
            }

            if !tuple_ids.is_empty() {
                result
                    .cell_data_mut()
                    .add_array(select_tuples(arr, &tuple_ids));
            }
        }
    }
    copy_active_attributes(grid.cell_data(), result.cell_data_mut());

    result
}

/// Extract a sub-region of a RectilinearGrid by coordinate ranges.
///
/// Finds the index ranges that best match the given coordinate bounds
/// and extracts that sub-grid.
pub fn extract_rectilinear_by_bounds(
    grid: &RectilinearGrid,
    x_range: (f64, f64),
    y_range: (f64, f64),
    z_range: (f64, f64),
) -> RectilinearGrid {
    let x_start = find_index(grid.x_coords(), x_range.0);
    let x_end = find_index(grid.x_coords(), x_range.1);
    let y_start = find_index(grid.y_coords(), y_range.0);
    let y_end = find_index(grid.y_coords(), y_range.1);
    let z_start = find_index(grid.z_coords(), z_range.0);
    let z_end = find_index(grid.z_coords(), z_range.1);

    extract_rectilinear_sub_grid(grid, (x_start, x_end), (y_start, y_end), (z_start, z_end))
}

fn find_index(coords: &[f64], value: f64) -> usize {
    match coords.binary_search_by(|c| c.partial_cmp(&value).unwrap_or(std::cmp::Ordering::Equal)) {
        Ok(i) => i,
        Err(i) => i.min(coords.len().saturating_sub(1)),
    }
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
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

    fn make_grid() -> RectilinearGrid {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 0.5, 1.0, 1.5, 2.0];
        let z = vec![0.0, 1.0];
        let mut grid = RectilinearGrid::from_coords(x, y, z);

        // Add point data
        let n = 5 * 5 * 2;
        let values: Vec<f64> = (0..n).map(|i| i as f64).collect();
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", values, 1)));

        grid
    }

    #[test]
    fn extract_sub_grid() {
        let grid = make_grid();
        let sub = extract_rectilinear_sub_grid(&grid, (1, 3), (0, 2), (0, 1));
        assert_eq!(sub.dimensions(), [3, 3, 2]);
        assert_eq!(sub.x_coords(), &[1.0, 2.0, 3.0]);
        assert_eq!(sub.y_coords(), &[0.0, 0.5, 1.0]);
    }

    #[test]
    fn extract_preserves_data() {
        let grid = make_grid();
        let sub = extract_rectilinear_sub_grid(&grid, (0, 4), (0, 4), (0, 1));
        assert_eq!(sub.dimensions(), grid.dimensions());
        let arr = sub.point_data().get_array("data");
        assert!(arr.is_some());
    }

    #[test]
    fn extract_by_bounds() {
        let grid = make_grid();
        let sub = extract_rectilinear_by_bounds(&grid, (0.5, 2.5), (0.0, 1.0), (0.0, 1.0));
        assert!(sub.dimensions()[0] >= 2);
        assert!(sub.dimensions()[1] >= 2);
    }

    #[test]
    fn extract_single_point() {
        let grid = make_grid();
        let sub = extract_rectilinear_sub_grid(&grid, (2, 2), (2, 2), (0, 0));
        assert_eq!(sub.dimensions(), [1, 1, 1]);
    }

    #[test]
    fn invalid_range() {
        let grid = make_grid();
        let sub = extract_rectilinear_sub_grid(&grid, (3, 1), (0, 0), (0, 0));
        // Should return empty grid
        assert_eq!(sub.dimensions()[0], 1); // default empty grid
    }
}
