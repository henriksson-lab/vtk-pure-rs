use crate::data::{AnyDataArray, DataArray, PolyData};

/// Bilateral scalar smoothing: smooth a scalar field while preserving edges.
///
/// Combines spatial and value-domain weights. Large value differences
/// get less smoothing, preserving discontinuities in the scalar field.
pub fn bilateral_scalar_smooth(
    input: &PolyData,
    array_name: &str,
    sigma_spatial: f64,
    sigma_value: f64,
    iterations: usize,
) -> PolyData {
    let n = input.points.len();
    if sigma_spatial <= 0.0
        || sigma_value <= 0.0
        || !sigma_spatial.is_finite()
        || !sigma_value.is_finite()
    {
        return input.clone();
    }
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return input.clone(),
    };

    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let inv_2ss = 1.0 / (2.0 * sigma_spatial * sigma_spatial);
    let inv_2sv = 1.0 / (2.0 * sigma_value * sigma_value);

    for _ in 0..iterations {
        let mut new_values = values.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let p = input.points.get(i);
            let vi = values[i];
            let mut sum = 0.0;
            let mut sum_w = 0.0;

            for &j in &neighbors[i] {
                let q = input.points.get(j);
                let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
                let dv = (values[j] - vi).powi(2);
                let w = (-d2 * inv_2ss - dv * inv_2sv).exp();
                sum += w * values[j];
                sum_w += w;
            }

            if sum_w > 1e-15 {
                new_values[i] = sum / sum_w;
            }
        }
        values = new_values;
    }

    let mut pd = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                array_name,
                values.clone(),
                1,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *pd.point_data_mut() = attrs;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooths_noise() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 0.0, 100.0],
                1,
            )));

        let result = bilateral_scalar_smooth(&pd, "s", 1.0, 200.0, 5); // large sigma_value -> smooth everything
        let arr = result.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] < 100.0, "val={}", buf[0]); // smoothed
    }

    #[test]
    fn preserves_step_edge() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 0.0, 0.0, 100.0, 100.0],
                1,
            )));

        let result = bilateral_scalar_smooth(&pd, "s", 1.0, 1.0, 5);
        let arr = result.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 30.0);
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 70.0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = bilateral_scalar_smooth(&pd, "nope", 1.0, 1.0, 5);
        assert_eq!(result.points.len(), 0);
    }
}
