use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute local neighborhood statistics for a scalar field.
///
/// For each vertex, computes min, max, mean, and range of the scalar
/// in its one-ring neighborhood. Adds "NeighborMin", "NeighborMax",
/// "NeighborMean", "NeighborRange" arrays.
pub fn neighborhood_stats(input: &PolyData, array_name: &str) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut nb_min = vec![0.0f64; n];
    let mut nb_max = vec![0.0f64; n];
    let mut nb_mean = vec![0.0f64; n];
    let mut nb_range = vec![0.0f64; n];

    for i in 0..n {
        let v = values[i];
        if neighbors[i].is_empty() {
            nb_min[i] = v;
            nb_max[i] = v;
            nb_mean[i] = v;
            continue;
        }
        let mut lo = v;
        let mut hi = v;
        let mut sum = v;
        for &j in &neighbors[i] {
            lo = lo.min(values[j]);
            hi = hi.max(values[j]);
            sum += values[j];
        }
        nb_min[i] = lo;
        nb_max[i] = hi;
        nb_mean[i] = sum / (neighbors[i].len() + 1) as f64;
        nb_range[i] = hi - lo;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NeighborMin",
            nb_min,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NeighborMax",
            nb_max,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NeighborMean",
            nb_mean,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NeighborRange",
            nb_range,
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
                "val",
                vec![0.0, 10.0, 5.0],
                1,
            )));

        let result = neighborhood_stats(&pd, "val");
        let min_arr = result.point_data().get_array("NeighborMin").unwrap();
        let max_arr = result.point_data().get_array("NeighborMax").unwrap();
        let mut buf = [0.0f64];
        min_arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        max_arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
    }

    #[test]
    fn uniform_zero_range() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![5.0; 3],
                1,
            )));

        let result = neighborhood_stats(&pd, "val");
        let range = result.point_data().get_array("NeighborRange").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            range.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = neighborhood_stats(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
