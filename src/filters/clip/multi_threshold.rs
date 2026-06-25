use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::Scalar;

/// A threshold interval with optional label.
#[derive(Debug, Clone)]
pub struct ThresholdInterval {
    pub min: f64,
    pub max: f64,
}

/// Extract cells matching any of multiple scalar threshold intervals.
///
/// For point scalars, a cell is kept when all of its point values fall in
/// one of the closed intervals, matching vtkMultiThreshold's allScalars
/// interval rule.
/// Adds a "ThresholdId" cell data array indicating which interval matched (0-indexed),
/// with first match winning when intervals overlap.
pub fn multi_threshold(
    input: &PolyData,
    scalars: &str,
    intervals: &[ThresholdInterval],
) -> PolyData {
    if intervals.is_empty() {
        return PolyData::new();
    }

    let scalar_arr = match input.point_data().get_array(scalars) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let n_pts = input.points.len();
    let mut scalar_data = vec![0.0f64; n_pts];
    let mut buf = [0.0f64];
    for (i, val) in scalar_data.iter_mut().enumerate() {
        scalar_arr.tuple_as_f64(i, &mut buf);
        *val = buf[0];
    }

    let mut out_polys = CellArray::new();
    let mut kept_cell_ids = Vec::new();
    let mut threshold_ids: Vec<f64> = Vec::new();
    let mut point_used = vec![false; n_pts];

    for (cell_id, cell) in input.polys.iter().enumerate() {
        if cell.is_empty() {
            continue;
        }

        let mut matched = None;
        for (i, interval) in intervals.iter().enumerate() {
            if cell.iter().all(|&id| {
                let value = scalar_data[id as usize];
                value >= interval.min && value <= interval.max
            }) {
                matched = Some(i);
                break;
            }
        }

        if let Some(id) = matched {
            out_polys.push_cell(cell);
            kept_cell_ids.push(cell_id);
            threshold_ids.push(id as f64);
            for &point_id in cell {
                point_used[point_id as usize] = true;
            }
        }
    }

    let point_ids: Vec<usize> = point_used
        .iter()
        .enumerate()
        .filter_map(|(id, used)| used.then_some(id))
        .collect();
    let mut point_map = vec![0i64; n_pts];
    let mut points = Points::new();
    for &point_id in &point_ids {
        point_map[point_id] = points.len() as i64;
        points.push(input.points.get(point_id));
    }

    let mut remapped_polys = CellArray::new();
    for cell in out_polys.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        remapped_polys.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = remapped_polys;
    for array in input.point_data().iter() {
        pd.point_data_mut()
            .add_array(copy_array_tuples(array, &point_ids));
    }
    pd.point_data_mut().set_active_scalars(scalars);
    for array in input.cell_data().iter() {
        pd.cell_data_mut()
            .add_array(copy_array_tuples(array, &kept_cell_ids));
    }
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ThresholdId",
            threshold_ids,
            1,
        )));
    pd
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

    fn make_test_mesh() -> PolyData {
        let mut pd = PolyData::new();
        // 4 vertices
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        // 2 triangles
        pd.polys.push_cell(&[0, 1, 2]); // scalar range [0, 2]
        pd.polys.push_cell(&[1, 2, 3]); // scalar range [1, 3]
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("val");
        pd
    }

    #[test]
    fn single_interval_matches_some() {
        let pd = make_test_mesh();
        let result = multi_threshold(&pd, "val", &[ThresholdInterval { min: 0.0, max: 2.0 }]);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn multiple_intervals() {
        let pd = make_test_mesh();
        let result = multi_threshold(
            &pd,
            "val",
            &[
                ThresholdInterval { min: 0.0, max: 1.2 },
                ThresholdInterval { min: 1.0, max: 3.0 },
            ],
        );
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn no_match() {
        let pd = make_test_mesh();
        let result = multi_threshold(
            &pd,
            "val",
            &[ThresholdInterval {
                min: 5.0,
                max: 10.0,
            }],
        );
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn missing_scalars() {
        let pd = make_test_mesh();
        let result = multi_threshold(
            &pd,
            "missing",
            &[ThresholdInterval {
                min: 0.0,
                max: 10.0,
            }],
        );
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn threshold_id_array() {
        let pd = make_test_mesh();
        let result = multi_threshold(
            &pd,
            "val",
            &[
                ThresholdInterval { min: 0.0, max: 1.2 },
                ThresholdInterval { min: 1.0, max: 3.0 },
            ],
        );
        let arr = result.cell_data().get_array("ThresholdId").unwrap();
        let mut ids = vec![0.0f64; 1];
        let mut b = [0.0f64];
        for i in 0..1 {
            arr.tuple_as_f64(i, &mut b);
            ids[i] = b[0];
        }
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 1.0); // second interval
    }
}
