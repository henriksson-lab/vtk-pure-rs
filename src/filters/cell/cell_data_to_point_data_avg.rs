use crate::data::{AnyDataArray, DataArray, PolyData};

/// Convert cell data to point data by averaging cell values at shared points.
///
/// For each point, averages the values from all cells that contain that point.
/// This is the inverse of point_data_to_cell_data.
pub fn cell_data_to_point_data(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = input.points.len();
    let num_comp = arr.num_components();
    let mut sums = vec![0.0f64; n * num_comp];
    let mut counts = vec![0usize; n];

    let mut buf = vec![0.0f64; num_comp];

    let mut cell_id = 0usize;
    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            arr.tuple_as_f64(cell_id, &mut buf);
            for &pid in cell.iter() {
                let idx = pid as usize;
                counts[idx] += 1;
                for c in 0..num_comp {
                    sums[idx * num_comp + c] += buf[c];
                }
            }
            cell_id += 1;
        }
    }

    // Average
    for i in 0..n {
        if counts[i] > 0 {
            let cnt = counts[i] as f64;
            for c in 0..num_comp {
                sums[i * num_comp + c] /= cnt;
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, sums, num_comp,
        )));
    pd
}

/// Convert point data to cell data by averaging point values per cell.
pub fn point_data_to_cell_data(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let num_comp = arr.num_components();
    let mut cell_values = Vec::new();
    let mut buf = vec![0.0f64; num_comp];

    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            let mut avg = vec![0.0f64; num_comp];
            for &pid in cell.iter() {
                arr.tuple_as_f64(pid as usize, &mut buf);
                for c in 0..num_comp {
                    avg[c] += buf[c];
                }
            }
            let cnt = cell.len() as f64;
            if cnt > 0.0 {
                for v in &mut avg {
                    *v /= cnt;
                }
            }
            cell_values.extend(avg);
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

    #[test]
    fn cell_to_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // cell 0, value=10
        pd.polys.push_cell(&[0, 2, 3]); // cell 1, value=20
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0],
                1,
            )));

        let result = cell_data_to_point_data(&pd, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        // Point 0: shared by cells 0,1 -> (10+20)/2 = 15
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 15.0);
        // Point 1: only cell 0 -> 10
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 10.0);
    }

    #[test]
    fn cell_to_point_uses_all_cell_arrays_in_vtk_order() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[1, 2, 0]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0],
                1,
            )));

        let result = cell_data_to_point_data(&pd, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 15.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 20.0);
    }

    #[test]
    fn point_to_cell() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let result = point_data_to_cell_data(&pd, "val");
        let arr = result.cell_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0); // (1+2+3)/3
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = cell_data_to_point_data(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
