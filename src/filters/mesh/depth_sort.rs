use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use std::cmp::Ordering;

/// Sort cells by depth along a specified vector.
///
/// Reorders cells so that cells farther along the vector come first,
/// matching vtkDepthSortPolyData's specified-vector back-to-front mode.
pub fn depth_sort(input: &PolyData, vector: [f64; 3]) -> PolyData {
    let cells = collect_cells(input);
    let mut indexed = indexed_depths(input, &cells, vector);

    // Sort far to near (for back-to-front rendering).
    indexed.sort_by(|a, b| compare_depth_desc(a.0, b.0).then_with(|| a.1.cmp(&b.1)));

    rebuild_sorted_poly_data(input, &cells, &indexed, true)
}

/// Sort cells front-to-back (for occlusion culling).
pub fn depth_sort_front_to_back(input: &PolyData, vector: [f64; 3]) -> PolyData {
    let cells = collect_cells(input);
    let mut indexed = indexed_depths(input, &cells, vector);

    indexed.sort_by(|a, b| compare_depth_asc(a.0, b.0).then_with(|| a.1.cmp(&b.1)));

    rebuild_sorted_poly_data(input, &cells, &indexed, false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Vert,
    Line,
    Poly,
    Strip,
}

#[derive(Debug, Clone)]
struct SortCell {
    kind: CellKind,
    point_ids: Vec<i64>,
    input_cell_id: usize,
}

fn collect_cells(input: &PolyData) -> Vec<SortCell> {
    let mut cells = Vec::with_capacity(input.total_cells());
    append_cells(&mut cells, CellKind::Vert, &input.verts);
    append_cells(&mut cells, CellKind::Line, &input.lines);
    append_cells(&mut cells, CellKind::Poly, &input.polys);
    append_cells(&mut cells, CellKind::Strip, &input.strips);
    cells
}

fn append_cells(cells: &mut Vec<SortCell>, kind: CellKind, cell_array: &CellArray) {
    for cell in cell_array.iter() {
        let input_cell_id = cells.len();
        cells.push(SortCell {
            kind,
            point_ids: cell.to_vec(),
            input_cell_id,
        });
    }
}

fn indexed_depths(input: &PolyData, cells: &[SortCell], vector: [f64; 3]) -> Vec<(f64, usize)> {
    cells
        .iter()
        .enumerate()
        .map(|(fi, cell)| (cell_depth(input, &cell.point_ids, vector), fi))
        .collect()
}

fn cell_depth(input: &PolyData, cell: &[i64], vector: [f64; 3]) -> f64 {
    let Some(&point_id) = cell.first() else {
        return 0.0;
    };
    let Some(idx) = valid_point_id(point_id, input.points.len()) else {
        return 0.0;
    };
    let point = input.points.get(idx);
    point[0] * vector[0] + point[1] * vector[1] + point[2] * vector[2]
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
    cells: &[SortCell],
    indexed: &[(f64, usize)],
    add_depth_array: bool,
) -> PolyData {
    let mut sorted_cells = Vec::with_capacity(indexed.len());
    for &(depth, cell_id) in indexed {
        sorted_cells.push((depth, &cells[cell_id]));
    }

    let mut out_verts = CellArray::new();
    let mut out_lines = CellArray::new();
    let mut out_polys = CellArray::new();
    let mut out_strips = CellArray::new();
    let output_order: Vec<usize> = sorted_cells
        .iter()
        .map(|(_, cell)| cell.input_cell_id)
        .collect();
    let depth_values: Vec<f64> = sorted_cells.iter().map(|(depth, _)| *depth).collect();

    append_sorted_kind(&sorted_cells, CellKind::Vert, &mut out_verts);
    append_sorted_kind(&sorted_cells, CellKind::Line, &mut out_lines);
    append_sorted_kind(&sorted_cells, CellKind::Poly, &mut out_polys);
    append_sorted_kind(&sorted_cells, CellKind::Strip, &mut out_strips);

    let mut pd = input.clone();
    pd.verts = out_verts;
    pd.lines = out_lines;
    pd.polys = out_polys;
    pd.strips = out_strips;
    reorder_cell_data(input, &mut pd, &output_order);
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

fn append_sorted_kind(sorted_cells: &[(f64, &SortCell)], kind: CellKind, output: &mut CellArray) {
    for &(_, cell) in sorted_cells {
        if cell.kind != kind {
            continue;
        }
        output.push_cell(&cell.point_ids);
    }
}

fn reorder_cell_data(input: &PolyData, output: &mut PolyData, output_order: &[usize]) {
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() != output_order.len() {
            continue;
        }
        output
            .cell_data_mut()
            .add_array(reorder_array(array, output_order));
    }
}

fn reorder_array(array: &AnyDataArray, output_order: &[usize]) -> AnyDataArray {
    macro_rules! reorder {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut values = Vec::with_capacity(output_order.len() * num_components);
            for &cell_id in output_order {
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
        pd.points.push([0.5, 1.0, 0.0]); // near along +z
        pd.points.push([0.0, 0.0, 10.0]);
        pd.points.push([1.0, 0.0, 10.0]);
        pd.points.push([0.5, 1.0, 10.0]); // far along +z
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = depth_sort(&pd, [0.0, 0.0, 1.0]);
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
        pd.points.push([0.0, 0.0, 10.0]);
        pd.points.push([1.0, 0.0, 10.0]);
        pd.points.push([0.5, 1.0, 10.0]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[0, 1, 2]); // far first, near second

        let result = depth_sort_front_to_back(&pd, [0.0, 0.0, 1.0]);
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
        pd.points.push([0.0, 0.0, 10.0]);
        pd.points.push([1.0, 0.0, 10.0]);
        pd.points.push([0.5, 1.0, 10.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("id", vec![7, 9], 1)));

        let result = depth_sort(&pd, [0.0, 0.0, 1.0]);
        let ids = result.cell_data().get_array("id").unwrap();
        let mut buf = [0.0f64];
        ids.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 9.0);
        ids.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 7.0);
    }

    #[test]
    fn sorts_non_polygon_cells_and_cell_data() {
        let mut pd = PolyData::new();
        for z in [
            0.0, 10.0, 1.0, 1.0, 11.0, 11.0, 2.0, 2.0, 2.0, 12.0, 12.0, 12.0,
        ] {
            pd.points.push([0.0, 0.0, z]);
        }
        pd.verts.push_cell(&[0]);
        pd.verts.push_cell(&[1]);
        pd.lines.push_cell(&[2, 3]);
        pd.lines.push_cell(&[4, 5]);
        pd.strips.push_cell(&[6, 7, 8]);
        pd.strips.push_cell(&[9, 10, 11]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "id",
                vec![1, 2, 3, 4, 5, 6],
                1,
            )));

        let result = depth_sort(&pd, [0.0, 0.0, 1.0]);

        assert_eq!(result.verts.cell(0), &[1]);
        assert_eq!(result.verts.cell(1), &[0]);
        assert_eq!(result.lines.cell(0), &[4, 5]);
        assert_eq!(result.lines.cell(1), &[2, 3]);
        assert_eq!(result.strips.cell(0), &[9, 10, 11]);
        assert_eq!(result.strips.cell(1), &[6, 7, 8]);

        let ids = result.cell_data().get_array("id").unwrap();
        let mut buf = [0.0f64];
        for (tuple, expected) in [6.0, 4.0, 2.0, 5.0, 3.0, 1.0].into_iter().enumerate() {
            ids.tuple_as_f64(tuple, &mut buf);
            assert_eq!(buf[0], expected);
        }
    }

    #[test]
    fn uses_first_point_depth_like_vtk_default() {
        let mut pd = PolyData::new();
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = depth_sort(&pd, [1.0, 0.0, 0.0]);
        let first = result.polys.cell(0);
        assert_eq!(first, &[0, 1, 2]);
    }
}
