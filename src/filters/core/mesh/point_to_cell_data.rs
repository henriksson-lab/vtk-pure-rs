use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Average point data values to cells.
///
/// For each cell (polygon), the output cell data value is the arithmetic mean
/// of the point data values at its vertices. Works for any named array with
/// any number of components.
pub fn point_to_cell_average(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let num_comp: usize = arr.num_components();
    let mut cell_values: Vec<f64> = Vec::new();
    let mut buf: Vec<f64> = vec![0.0; num_comp];

    append_cell_averages(&input.verts, arr, &mut buf, &mut cell_values);
    append_cell_averages(&input.lines, arr, &mut buf, &mut cell_values);
    append_cell_averages(&input.polys, arr, &mut buf, &mut cell_values);
    append_cell_averages(&input.strips, arr, &mut buf, &mut cell_values);

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            cell_values,
            num_comp,
        )));
    if input
        .point_data()
        .scalars()
        .is_some_and(|scalars| scalars.name() == array_name)
    {
        pd.cell_data_mut().set_active_scalars(array_name);
    }
    pd
}

fn append_cell_averages(
    cells: &CellArray,
    arr: &AnyDataArray,
    buf: &mut [f64],
    cell_values: &mut Vec<f64>,
) {
    let num_comp = arr.num_components();
    for cell in cells.iter() {
        let mut avg: Vec<f64> = vec![0.0; num_comp];
        let mut count = 0usize;
        for &pid in cell.iter() {
            let Some(pid) = valid_tuple_id(pid, arr.num_tuples()) else {
                continue;
            };
            arr.tuple_as_f64(pid, buf);
            for c in 0..num_comp {
                avg[c] += buf[c];
            }
            count += 1;
        }
        if count == 0 {
            cell_values.extend(std::iter::repeat_n(0.0, num_comp));
            continue;
        }
        let cnt = count as f64;
        cell_values.extend(avg.into_iter().map(|v| v / cnt));
    }
}

fn valid_tuple_id(id: i64, num_tuples: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < num_tuples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, PolyData};

    #[test]
    fn scalar_average() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "height",
                vec![3.0, 6.0, 9.0],
                1,
            )));

        let result = point_to_cell_average(&pd, "height");
        let arr = result.cell_data().get_array("height").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn vector_average() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        // 3-component vector data
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vel",
                vec![
                    1.0, 0.0, 0.0, // pt0
                    0.0, 2.0, 0.0, // pt1
                    0.0, 0.0, 3.0, // pt2
                ],
                3,
            )));

        let result = point_to_cell_average(&pd, "vel");
        let arr = result.cell_data().get_array("vel").unwrap();
        assert_eq!(arr.num_components(), 3);
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        let expected: [f64; 3] = [1.0 / 3.0, 2.0 / 3.0, 1.0];
        for i in 0..3 {
            assert!((buf[i] - expected[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn missing_array_returns_clone() {
        let pd = PolyData::new();
        let result = point_to_cell_average(&pd, "nonexistent");
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn malformed_cell_ids_are_ignored() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, -1, 99, 1]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "height",
                vec![4.0, 8.0],
                1,
            )));

        let result = point_to_cell_average(&pd, "height");
        let arr = result.cell_data().get_array("height").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn averages_all_poly_data_cell_arrays_in_vtk_order() {
        let mut pd = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[1, 2]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.strips.push_cell(&[1, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "height",
                vec![2.0, 4.0, 8.0, 16.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("height");

        let result = point_to_cell_average(&pd, "height");
        let arr = result.cell_data().get_array("height").unwrap();
        assert_eq!(arr.num_tuples(), 4);
        assert!(result.cell_data().scalars().is_some());

        let mut buf = [0.0f64];
        let expected = [2.0, 6.0, 14.0 / 3.0, 28.0 / 3.0];
        for (i, expected) in expected.into_iter().enumerate() {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - expected).abs() < 1e-10);
        }
    }
}
