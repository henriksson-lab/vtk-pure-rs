use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-cell statistics from adjacent point data.
///
/// For each cell, computes min, max, mean, and range of the named
/// point data array over the cell's vertices. Adds cell data arrays.
pub fn cell_from_point_stats(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let mut buf = [0.0f64];

    let mut cell_min = Vec::new();
    let mut cell_max = Vec::new();
    let mut cell_mean = Vec::new();
    let mut cell_range = Vec::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            cell_min.push(0.0);
            cell_max.push(0.0);
            cell_mean.push(0.0);
            cell_range.push(0.0);
            continue;
        }
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        let mut sum = 0.0;
        for &id in cell.iter() {
            arr.tuple_as_f64(id as usize, &mut buf);
            lo = lo.min(buf[0]);
            hi = hi.max(buf[0]);
            sum += buf[0];
        }
        let n = cell.len() as f64;
        cell_min.push(lo);
        cell_max.push(hi);
        cell_mean.push(sum / n);
        cell_range.push(hi - lo);
    }

    let prefix = array_name;
    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{}_CellMin", prefix),
            cell_min,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{}_CellMax", prefix),
            cell_max,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{}_CellMean", prefix),
            cell_mean,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &format!("{}_CellRange", prefix),
            cell_range,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0, 30.0],
                1,
            )));

        let result = cell_from_point_stats(&pd, "temp");
        let min_a = result.cell_data().get_array("temp_CellMin").unwrap();
        let max_a = result.cell_data().get_array("temp_CellMax").unwrap();
        let mean_a = result.cell_data().get_array("temp_CellMean").unwrap();
        let range_a = result.cell_data().get_array("temp_CellRange").unwrap();
        let mut buf = [0.0f64];
        min_a.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        max_a.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 30.0);
        mean_a.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 20.0);
        range_a.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 20.0);
    }

    #[test]
    fn uniform_zero_range() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 3], 1)));

        let result = cell_from_point_stats(&pd, "v");
        let range_a = result.cell_data().get_array("v_CellRange").unwrap();
        let mut buf = [0.0f64];
        range_a.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = cell_from_point_stats(&pd, "nope");
        assert_eq!(result.polys.num_cells(), 0);
    }
}
