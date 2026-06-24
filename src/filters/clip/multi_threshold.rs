use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// A threshold interval with optional label.
#[derive(Debug, Clone)]
pub struct ThresholdInterval {
    pub min: f64,
    pub max: f64,
}

/// Extract cells matching any of multiple scalar threshold intervals.
///
/// For each triangle, evaluates the mean scalar value at its vertices and
/// keeps the cell if it falls within any of the provided intervals.
/// Adds a "ThresholdId" cell data array indicating which interval matched (0-indexed),
/// or -1 if the cell matched multiple intervals (first match wins).
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
    let mut threshold_ids: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }

        // Mean scalar over the cell
        let mean: f64 =
            cell.iter().map(|&id| scalar_data[id as usize]).sum::<f64>() / cell.len() as f64;

        // Check which interval matches
        let mut matched = None;
        for (i, interval) in intervals.iter().enumerate() {
            if mean >= interval.min && mean <= interval.max {
                matched = Some(i);
                break;
            }
        }

        if let Some(id) = matched {
            out_polys.push_cell(cell);
            threshold_ids.push(id as f64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.polys = out_polys;
    // Copy point data
    for i in 0..input.point_data().num_arrays() {
        pd.point_data_mut()
            .add_array(input.point_data().get_array_by_index(i).unwrap().clone());
    }
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ThresholdId",
            threshold_ids,
            1,
        )));
    pd
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
        pd.polys.push_cell(&[0, 1, 2]); // mean scalar = (0+1+2)/3 = 1.0
        pd.polys.push_cell(&[0, 2, 3]); // mean scalar = (0+2+3)/3 = 1.67
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        pd
    }

    #[test]
    fn single_interval_matches_some() {
        let pd = make_test_mesh();
        let result = multi_threshold(&pd, "val", &[ThresholdInterval { min: 0.0, max: 1.2 }]);
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
                ThresholdInterval { min: 1.5, max: 2.0 },
            ],
        );
        assert_eq!(result.polys.num_cells(), 2);
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
                ThresholdInterval { min: 1.5, max: 2.0 },
            ],
        );
        let arr = result.cell_data().get_array("ThresholdId").unwrap();
        let mut ids = vec![0.0f64; 2];
        let mut b = [0.0f64];
        for i in 0..2 {
            arr.tuple_as_f64(i, &mut b);
            ids[i] = b[0];
        }
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], 0.0); // first interval
        assert_eq!(ids[1], 1.0); // second interval
    }
}
