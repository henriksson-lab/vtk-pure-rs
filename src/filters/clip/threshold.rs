use crate::data::{CellArray, DataArray, DataSet, Points, PolyData};

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
    let mut kept_cells: Vec<Vec<i64>> = Vec::new();

    for cell in input.polys.iter() {
        let all_in_range = cell.iter().all(|&id| {
            let v = values[id as usize];
            v >= lower && v <= upper
        });
        if all_in_range {
            for &id in cell {
                point_used[id as usize] = true;
            }
            kept_cells.push(cell.to_vec());
        }
    }

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

    // Remap cells
    let mut polys = CellArray::new();
    for cell in &kept_cells {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        polys.push_cell(&remapped);
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.polys = polys;
    output.point_data_mut().add_array(new_scalars.into());
    output.point_data_mut().set_active_scalars(scalars.name());
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
}
