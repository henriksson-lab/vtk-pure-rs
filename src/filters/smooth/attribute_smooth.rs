use crate::data::{AnyDataArray, DataArray, PolyData};

/// Smooth point data attributes over the mesh connectivity.
///
/// For each point, replaces its scalar value with the average of its
/// one-ring neighborhood (connected vertices). Repeats for `iterations`.
/// Operates on the named scalar array, preserving all other arrays.
pub fn attribute_smooth(input: &PolyData, array_name: &str, iterations: usize) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    // Read initial values
    let num_comp = arr.num_components();
    let mut values = vec![0.0f64; n * num_comp];
    let mut buf = vec![0.0f64; num_comp];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        for c in 0..num_comp {
            values[i * num_comp + c] = buf[c];
        }
    }

    // Build adjacency from polygon cells
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            neighbors[a].push(b);
            neighbors[b].push(a);
        }
    }
    // Also from line cells
    for cell in input.lines.iter() {
        for i in 0..cell.len() - 1 {
            let a = cell[i] as usize;
            let b = cell[i + 1] as usize;
            neighbors[a].push(b);
            neighbors[b].push(a);
        }
    }
    // Deduplicate neighbors
    for nbrs in &mut neighbors {
        nbrs.sort();
        nbrs.dedup();
    }

    // Iterative smoothing
    for _ in 0..iterations {
        let mut new_values = vec![0.0f64; n * num_comp];
        for i in 0..n {
            if neighbors[i].is_empty() {
                for c in 0..num_comp {
                    new_values[i * num_comp + c] = values[i * num_comp + c];
                }
            } else {
                let count = neighbors[i].len() as f64 + 1.0;
                for c in 0..num_comp {
                    let mut sum = values[i * num_comp + c];
                    for &j in &neighbors[i] {
                        sum += values[j * num_comp + c];
                    }
                    new_values[i * num_comp + c] = sum / count;
                }
            }
        }
        values = new_values;
    }

    let mut pd = input.clone();
    let smoothed = AnyDataArray::F64(DataArray::from_vec(array_name, values, num_comp));

    // Replace the array
    // Remove old and add new
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
    fn smooth_spike() {
        let mut pd = PolyData::new();
        // Triangle with center spike value
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![0.0, 0.0, 10.0],
                1,
            )));

        let result = attribute_smooth(&pd, "temp", 1);
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];

        // Point 2 was 10.0, neighbors are 0 and 1 (both 0.0)
        // New value = (10 + 0 + 0) / 3 = 3.33...
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 10.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn zero_iterations() {
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

        let result = attribute_smooth(&pd, "val", 0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 3.0);
    }

    #[test]
    fn missing_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = attribute_smooth(&pd, "nonexistent", 5);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn convergence() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 9.0],
                1,
            )));

        // Many iterations should converge to the mean (3.0)
        let result = attribute_smooth(&pd, "v", 100);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 3.0).abs() < 0.01, "val[{}]={}", i, buf[0]);
        }
    }
}
