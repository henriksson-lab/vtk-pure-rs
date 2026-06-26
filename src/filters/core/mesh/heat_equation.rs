use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};

/// Solve the heat equation on a triangle mesh surface.
///
/// Diffuses a scalar field over the mesh using explicit Euler integration.
/// At each iteration, each vertex value moves toward the average of its
/// neighbors by an amount controlled by `dt`.
///
/// `dt` is clamped to [0, 0.5] for stability of the explicit scheme.
pub fn heat_diffuse(input: &PolyData, array_name: &str, dt: f64, iterations: usize) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = input.points.len();
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    // Build adjacency from polygon connectivity
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        let cn: usize = cell.len();
        for i in 0..cn {
            let a: usize = cell[i] as usize;
            let b: usize = cell[(i + 1) % cn] as usize;
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

    let dt_clamped: f64 = dt.clamp(0.0, 0.5);

    // Explicit Euler integration
    for _ in 0..iterations {
        let mut new_values: Vec<f64> = values.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let neighbor_count: f64 = neighbors[i].len() as f64;
            let avg: f64 = neighbors[i].iter().map(|&j| values[j]).sum::<f64>() / neighbor_count;
            new_values[i] = values[i] + dt_clamped * (avg - values[i]);
        }
        values = new_values;
    }

    // Build output with updated array
    let mut pd = input.clone();
    let mut attrs = DataSetAttributes::new();
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
    pd.point_data_mut().set_active_scalars(array_name);
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_spreads_from_hot_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let initial: Vec<f64> = vec![100.0, 0.0, 0.0];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Temperature",
                initial,
                1,
            )));

        let result = heat_diffuse(&pd, "Temperature", 0.5, 10);
        let out_arr = result.point_data().get_array("Temperature").unwrap();
        let mut buf = [0.0f64];

        // Hot vertex should have cooled down
        out_arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 100.0);

        // Cold vertices should have warmed up
        out_arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 0.0);
        out_arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn converges_to_equilibrium() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let initial: Vec<f64> = vec![90.0, 0.0, 0.0];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("T", initial, 1)));

        let result = heat_diffuse(&pd, "T", 0.5, 2000);
        let out_arr = result.point_data().get_array("T").unwrap();
        let mut buf = [0.0f64];

        let mut vals: Vec<f64> = Vec::new();
        for i in 0..3 {
            out_arr.tuple_as_f64(i, &mut buf);
            vals.push(buf[0]);
        }
        // All vertices should converge to ~30.0 (mean of 90, 0, 0)
        assert!((vals[0] - vals[1]).abs() < 1.0);
        assert!((vals[1] - vals[2]).abs() < 1.0);
        assert!((vals[0] - 30.0).abs() < 1.0);
    }

    #[test]
    fn missing_array_returns_clone() {
        let pd = PolyData::new();
        let result = heat_diffuse(&pd, "missing", 0.1, 5);
        assert_eq!(result.points.len(), 0);
    }
}
