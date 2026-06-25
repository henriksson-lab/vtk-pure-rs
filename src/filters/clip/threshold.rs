use crate::data::{AnyDataArray, CellArray, DataArray, DataSet, Points, PolyData};
use crate::types::Scalar;

/// Extract cells whose scalar values fall within [lower, upper].
///
/// Uses the active scalars from point data. A cell is kept if ALL of its
/// vertices have scalar values within the range.
pub fn threshold(input: &PolyData, lower: f64, upper: f64) -> PolyData {
    let scalars = match input.point_data().scalars() {
        Some(s) => s,
        None => return PolyData::new(),
    };

    // Read scalar values into a flat array
    let n = input.num_points();
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (i, val) in values.iter_mut().enumerate() {
        scalars.tuple_as_f64(i, &mut buf);
        *val = buf[0];
    }

    // Track which points are used
    let mut point_used = vec![false; n];
    let mut kept_cells = KeptCells::default();

    collect_kept_cells(
        &input.verts,
        CellKind::Verts,
        0,
        &values,
        lower,
        upper,
        &mut point_used,
        &mut kept_cells,
    );
    let lines_offset = input.verts.num_cells();
    collect_kept_cells(
        &input.lines,
        CellKind::Lines,
        lines_offset,
        &values,
        lower,
        upper,
        &mut point_used,
        &mut kept_cells,
    );
    let polys_offset = lines_offset + input.lines.num_cells();
    collect_kept_cells(
        &input.polys,
        CellKind::Polys,
        polys_offset,
        &values,
        lower,
        upper,
        &mut point_used,
        &mut kept_cells,
    );
    let strips_offset = polys_offset + input.polys.num_cells();
    collect_kept_cells(
        &input.strips,
        CellKind::Strips,
        strips_offset,
        &values,
        lower,
        upper,
        &mut point_used,
        &mut kept_cells,
    );

    // Build compact point map
    let mut point_map = vec![0i64; n];
    let mut new_points = Points::<f64>::new();
    let mut new_scalars = DataArray::<f64>::new(scalars.name(), 1);

    for (old_id, &used) in point_used.iter().enumerate() {
        if used {
            point_map[old_id] = new_points.len() as i64;
            new_points.push(input.points.get(old_id));
            new_scalars.push_tuple(&[values[old_id]]);
        }
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.verts = remap_kept_cells(&kept_cells.verts, &point_map);
    output.lines = remap_kept_cells(&kept_cells.lines, &point_map);
    output.polys = remap_kept_cells(&kept_cells.polys, &point_map);
    output.strips = remap_kept_cells(&kept_cells.strips, &point_map);
    copy_point_data_for_used_points(input, &point_used, &mut output);
    if !output.point_data().has_array(scalars.name()) {
        output.point_data_mut().add_array(new_scalars.into());
    }
    output.point_data_mut().set_active_scalars(scalars.name());
    copy_cell_data_for_kept_cells(input, &kept_cells.cell_ids, &mut output);
    output
}

/// Extract cells whose scalar values are above the given threshold.
pub fn threshold_above(input: &PolyData, value: f64) -> PolyData {
    threshold(input, value, f64::INFINITY)
}

/// Extract cells whose scalar values are below the given threshold.
pub fn threshold_below(input: &PolyData, value: f64) -> PolyData {
    threshold(input, f64::NEG_INFINITY, value)
}

#[derive(Clone, Copy)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

#[derive(Default)]
struct KeptCells {
    verts: Vec<Vec<i64>>,
    lines: Vec<Vec<i64>>,
    polys: Vec<Vec<i64>>,
    strips: Vec<Vec<i64>>,
    cell_ids: Vec<usize>,
}

impl KeptCells {
    fn push(&mut self, kind: CellKind, cell: Vec<i64>, cell_id: usize) {
        match kind {
            CellKind::Verts => self.verts.push(cell),
            CellKind::Lines => self.lines.push(cell),
            CellKind::Polys => self.polys.push(cell),
            CellKind::Strips => self.strips.push(cell),
        }
        self.cell_ids.push(cell_id);
    }
}

fn collect_kept_cells(
    cells: &CellArray,
    kind: CellKind,
    cell_offset: usize,
    values: &[f64],
    lower: f64,
    upper: f64,
    point_used: &mut [bool],
    kept_cells: &mut KeptCells,
) {
    for (cell_id, cell) in cells.iter().enumerate() {
        let all_in_range = cell.iter().all(|&id| {
            let v = values[id as usize];
            v >= lower && v <= upper
        });
        if all_in_range {
            for &id in cell {
                point_used[id as usize] = true;
            }
            kept_cells.push(kind, cell.to_vec(), cell_offset + cell_id);
        }
    }
}

fn remap_kept_cells(cells: &[Vec<i64>], point_map: &[i64]) -> CellArray {
    let mut output = CellArray::new();
    for cell in cells {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        output.push_cell(&remapped);
    }
    output
}

fn copy_point_data_for_used_points(input: &PolyData, point_used: &[bool], output: &mut PolyData) {
    let point_ids: Vec<usize> = point_used
        .iter()
        .enumerate()
        .filter_map(|(id, used)| used.then_some(id))
        .collect();
    for array in input.point_data().iter() {
        output
            .point_data_mut()
            .add_array(copy_array_tuples(array, &point_ids));
    }
}

fn copy_cell_data_for_kept_cells(input: &PolyData, cell_ids: &[usize], output: &mut PolyData) {
    for array in input.cell_data().iter() {
        output
            .cell_data_mut()
            .add_array(copy_array_tuples(array, cell_ids));
    }
}

fn copy_array_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy_typed {
        ($arr:expr, $variant:path) => {{
            $variant(copy_typed_array($arr, tuple_ids))
        }};
    }

    match array {
        AnyDataArray::F32(a) => copy_typed!(a, AnyDataArray::F32),
        AnyDataArray::F64(a) => copy_typed!(a, AnyDataArray::F64),
        AnyDataArray::I8(a) => copy_typed!(a, AnyDataArray::I8),
        AnyDataArray::I16(a) => copy_typed!(a, AnyDataArray::I16),
        AnyDataArray::I32(a) => copy_typed!(a, AnyDataArray::I32),
        AnyDataArray::I64(a) => copy_typed!(a, AnyDataArray::I64),
        AnyDataArray::U8(a) => copy_typed!(a, AnyDataArray::U8),
        AnyDataArray::U16(a) => copy_typed!(a, AnyDataArray::U16),
        AnyDataArray::U32(a) => copy_typed!(a, AnyDataArray::U32),
        AnyDataArray::U64(a) => copy_typed!(a, AnyDataArray::U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, tuple_ids: &[usize]) -> DataArray<T> {
    let mut output = DataArray::new(array.name(), array.num_components());
    for &tuple_id in tuple_ids {
        output.push_tuple(array.tuple(tuple_id));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_data() -> PolyData {
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        // Scalars: 0, 5, 10, 15
        let scalars = DataArray::from_vec("temp", vec![0.0, 5.0, 10.0, 15.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("temp");
        pd
    }

    #[test]
    fn threshold_range() {
        let pd = make_test_data();
        // Only first triangle has all points in [0, 10]
        let result = threshold(&pd, 0.0, 10.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn threshold_keeps_all() {
        let pd = make_test_data();
        let result = threshold(&pd, -100.0, 100.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn threshold_removes_all() {
        let pd = make_test_data();
        let result = threshold(&pd, 100.0, 200.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn threshold_preserves_vtk_polydata_cell_arrays() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[1, 2]);
        pd.polys.push_cell(&[0, 3, 4]);
        pd.strips.push_cell(&[3, 4, 5]);
        let scalars = DataArray::from_vec("temp", vec![5.0, 5.0, 50.0, 5.0, 5.0, 5.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("temp");
        pd.cell_data_mut()
            .add_array(DataArray::from_vec("cell_id", vec![10, 20, 30, 40], 1).into());

        let result = threshold(&pd, 0.0, 10.0);

        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.lines.num_cells(), 0);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
        let cell_ids = result
            .cell_data()
            .get_array("cell_id")
            .unwrap()
            .to_f64_vec();
        assert_eq!(cell_ids, vec![10.0, 30.0, 40.0]);
    }
}
