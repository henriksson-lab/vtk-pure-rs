use crate::data::{AnyDataArray, DataArray, PolyData};

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

    for cell in input.polys.iter() {
        let mut avg: Vec<f64> = vec![0.0; num_comp];
        let cell_len: usize = cell.len();
        if cell_len == 0 {
            for _ in 0..num_comp {
                cell_values.push(0.0);
            }
            continue;
        }
        for &pid in cell.iter() {
            arr.tuple_as_f64(pid as usize, &mut buf);
            for c in 0..num_comp {
                avg[c] += buf[c];
            }
        }
        let cnt: f64 = cell_len as f64;
        for c in 0..num_comp {
            cell_values.push(avg[c] / cnt);
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            cell_values,
            num_comp,
        )));
    pd
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
}
