use crate::data::{AnyDataArray, DataArray, PolyData};

/// Convert a cell data array to point data by averaging.
///
/// For each point, finds all cells that contain it and averages the cell data
/// values from those cells. The resulting array is added to point data with
/// the same name.
pub fn cell_to_point_average(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let num_components: usize = arr.num_components();
    let num_points: usize = input.points.len();

    // For each point, accumulate values and count contributions
    let mut accum: Vec<f64> = vec![0.0; num_points * num_components];
    let mut counts: Vec<u32> = vec![0; num_points];

    let mut cell_val: Vec<f64> = vec![0.0; num_components];

    let mut cell_idx = 0;
    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            if cell_idx >= arr.num_tuples() {
                break;
            }
            arr.tuple_as_f64(cell_idx, &mut cell_val);
            for &pt_id in cell {
                if pt_id < 0 || pt_id as usize >= num_points {
                    continue;
                }
                let pid: usize = pt_id as usize;
                counts[pid] += 1;
                for c in 0..num_components {
                    accum[pid * num_components + c] += cell_val[c];
                }
            }
            cell_idx += 1;
        }
    }

    // Average
    let mut result: Vec<f64> = vec![0.0; num_points * num_components];
    for i in 0..num_points {
        if counts[i] > 0 {
            let cnt: f64 = counts[i] as f64;
            for c in 0..num_components {
                result[i * num_components + c] = accum[i * num_components + c] / cnt;
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            result,
            num_components,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_scalar() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pd = pd;
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("temp", vec![6.0], 1)));

        let result = cell_to_point_average(&pd, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        // All three points belong to the single cell, so each gets 6.0
        let mut val = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut val);
            assert!((val[0] - 6.0).abs() < 1e-10);
        }
    }

    #[test]
    fn shared_vertex_averages() {
        // Two triangles sharing vertex 1
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let mut pd = pd;
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![10.0, 20.0],
                1,
            )));

        let result = cell_to_point_average(&pd, "val");
        let arr = result.point_data().get_array("val").unwrap();
        // Point 0 is in cell 0 only -> 10.0
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 10.0).abs() < 1e-10);
        // Point 1 is in both cells -> (10+20)/2 = 15.0
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 15.0).abs() < 1e-10);
        // Point 3 is in cell 1 only -> 20.0
        arr.tuple_as_f64(3, &mut val);
        assert!((val[0] - 20.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = cell_to_point_average(&pd, "nonexistent");
        assert_eq!(result.points.len(), 3);
    }
}
