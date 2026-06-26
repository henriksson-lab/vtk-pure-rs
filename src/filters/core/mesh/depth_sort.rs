use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use std::cmp::Ordering;

/// Sort faces by depth (distance from a viewpoint).
///
/// Reorders cells so that faces farther from the viewpoint come first.
/// Useful for transparency rendering (painter's algorithm).
pub fn depth_sort(input: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let mut indexed = indexed_depths(input, &cells, viewpoint);

    // Sort far to near (for back-to-front rendering).
    indexed.sort_by(|a, b| compare_depth_desc(a.0, b.0).then_with(|| a.1.cmp(&b.1)));

    rebuild_sorted_poly_data(input, &cells, &indexed, true)
}

/// Sort faces front-to-back (for occlusion culling).
pub fn depth_sort_front_to_back(input: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let mut indexed = indexed_depths(input, &cells, viewpoint);

    indexed.sort_by(|a, b| compare_depth_asc(a.0, b.0).then_with(|| a.1.cmp(&b.1)));

    rebuild_sorted_poly_data(input, &cells, &indexed, false)
}

fn indexed_depths(input: &PolyData, cells: &[Vec<i64>], viewpoint: [f64; 3]) -> Vec<(f64, usize)> {
    cells
        .iter()
        .enumerate()
        .map(|(fi, cell)| (cell_depth(input, cell, viewpoint), fi))
        .collect()
}

fn cell_depth(input: &PolyData, cell: &[i64], viewpoint: [f64; 3]) -> f64 {
    let mut centroid = [0.0; 3];
    let mut count = 0usize;
    for &point_id in cell {
        let Some(idx) = valid_point_id(point_id, input.points.len()) else {
            continue;
        };
        let point = input.points.get(idx);
        centroid[0] += point[0];
        centroid[1] += point[1];
        centroid[2] += point[2];
        count += 1;
    }

    if count == 0 {
        return 0.0;
    }

    let inv_count = 1.0 / count as f64;
    centroid[0] *= inv_count;
    centroid[1] *= inv_count;
    centroid[2] *= inv_count;

    (centroid[0] - viewpoint[0]).powi(2)
        + (centroid[1] - viewpoint[1]).powi(2)
        + (centroid[2] - viewpoint[2]).powi(2)
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&idx| idx < n_points)
}

fn compare_depth_desc(a: f64, b: f64) -> Ordering {
    b.partial_cmp(&a).unwrap_or(Ordering::Equal)
}

fn compare_depth_asc(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

fn rebuild_sorted_poly_data(
    input: &PolyData,
    cells: &[Vec<i64>],
    indexed: &[(f64, usize)],
    add_depth_array: bool,
) -> PolyData {
    let mut out_polys = CellArray::new();
    let mut depth_values = Vec::with_capacity(indexed.len());
    for &(depth, cell_id) in indexed {
        out_polys.push_cell(&cells[cell_id]);
        depth_values.push(depth.sqrt());
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    reorder_cell_data(input, &mut pd, indexed);
    if add_depth_array {
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Depth",
                depth_values,
                1,
            )));
    }
    pd
}

fn reorder_cell_data(input: &PolyData, output: &mut PolyData, indexed: &[(f64, usize)]) {
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() != indexed.len() {
            continue;
        }
        output
            .cell_data_mut()
            .add_array(reorder_array(array, indexed));
    }
}

fn reorder_array(array: &AnyDataArray, indexed: &[(f64, usize)]) -> AnyDataArray {
    macro_rules! reorder {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut values = Vec::with_capacity(indexed.len() * num_components);
            for &(_, cell_id) in indexed {
                values.extend_from_slice($array.tuple(cell_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), values, num_components))
        }};
    }

    match array {
        AnyDataArray::F32(array) => reorder!(array, F32),
        AnyDataArray::F64(array) => reorder!(array, F64),
        AnyDataArray::I8(array) => reorder!(array, I8),
        AnyDataArray::I16(array) => reorder!(array, I16),
        AnyDataArray::I32(array) => reorder!(array, I32),
        AnyDataArray::I64(array) => reorder!(array, I64),
        AnyDataArray::U8(array) => reorder!(array, U8),
        AnyDataArray::U16(array) => reorder!(array, U16),
        AnyDataArray::U32(array) => reorder!(array, U32),
        AnyDataArray::U64(array) => reorder!(array, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_to_front() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]); // near
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]); // far
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = depth_sort(&pd, [0.0, 0.0, 0.0]);
        // Far face should come first
        let arr = result.cell_data().get_array("Depth").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let d0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let d1 = buf[0];
        assert!(d0 >= d1); // first cell is farther
    }

    #[test]
    fn front_to_back() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[0, 1, 2]); // far first, near second

        let result = depth_sort_front_to_back(&pd, [0.0, 0.0, 0.0]);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn single_face() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = depth_sort(&pd, [0.0, 0.0, 5.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(depth_sort(&pd, [0.0; 3]).polys.num_cells(), 0);
    }

    #[test]
    fn cell_data_follows_sorted_polys() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("id", vec![7, 9], 1)));

        let result = depth_sort(&pd, [0.0, 0.0, 0.0]);
        let ids = result.cell_data().get_array("id").unwrap();
        let mut buf = [0.0f64];
        ids.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 9.0);
        ids.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 7.0);
    }
}
