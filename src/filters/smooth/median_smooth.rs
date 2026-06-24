use crate::data::{AnyDataArray, DataArray, PolyData};

/// Median smoothing of a scalar point data array.
///
/// For each point, replaces its value with the median of its one-ring
/// neighborhood values. More robust than Laplacian smoothing against outliers.
/// Repeats for `iterations`.
pub fn median_smooth(input: &PolyData, array_name: &str, iterations: usize) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (i, v) in values.iter_mut().enumerate() {
        arr.tuple_as_f64(i, &mut buf);
        *v = buf[0];
    }

    // Build adjacency
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            neighbors[a].push(b);
            neighbors[b].push(a);
        }
    }
    for nbrs in &mut neighbors {
        nbrs.sort();
        nbrs.dedup();
    }

    for _ in 0..iterations {
        let mut new_values = vec![0.0f64; n];
        for i in 0..n {
            let mut ring: Vec<f64> = vec![values[i]];
            for &j in &neighbors[i] {
                ring.push(values[j]);
            }
            ring.sort_by(|a, b| a.partial_cmp(b).unwrap());
            new_values[i] = ring[ring.len() / 2];
        }
        values = new_values;
    }

    let mut pd = input.clone();
    let smoothed = AnyDataArray::F64(DataArray::from_vec(array_name, values, 1));

    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            new_attrs.add_array(smoothed.clone());
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *pd.point_data_mut() = new_attrs;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_spike() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        // Point 2 has an outlier value
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 1.0, 100.0],
                1,
            )));

        let result = median_smooth(&pd, "val", 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        // Median of {100, 1, 1} = 1.0
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn uniform_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![5.0, 5.0, 5.0],
                1,
            )));

        let result = median_smooth(&pd, "val", 3);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 5.0);
        }
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = median_smooth(&pd, "nope", 1);
        assert_eq!(result.points.len(), 0);
    }
}
