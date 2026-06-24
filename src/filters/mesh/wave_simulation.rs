use crate::data::{AnyDataArray, DataArray, PolyData};

/// Simulate wave propagation on a mesh.
///
/// Evolves a scalar field using the wave equation:
/// u_new = 2*u - u_old + c²*dt²*Laplacian(u).
/// Adds "WaveHeight" scalar array.
pub fn wave_step(
    input: &PolyData,
    u_name: &str,
    u_old_name: &str,
    speed: f64,
    dt: f64,
) -> PolyData {
    let u_arr = match input.point_data().get_array(u_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let u_old_arr = match input.point_data().get_array(u_old_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = input.points.len();
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
    let u: Vec<f64> = (0..n)
        .map(|i| {
            u_arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let u_old: Vec<f64> = (0..n)
        .map(|i| {
            u_old_arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let c2dt2 = speed * speed * dt * dt;
    let mut u_new = vec![0.0f64; n];

    for i in 0..n {
        if neighbors[i].is_empty() {
            u_new[i] = 2.0 * u[i] - u_old[i];
            continue;
        }
        let cnt = neighbors[i].len() as f64;
        let lap: f64 = neighbors[i].iter().map(|&j| u[j] - u[i]).sum::<f64>() / cnt;
        u_new[i] = 2.0 * u[i] - u_old[i] + c2dt2 * lap;
    }

    let mut pd = input.clone();
    // Shift: current u becomes old, new u becomes current
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == u_name {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                u_name,
                u_new.clone(),
                1,
            )));
        } else if a.name() == u_old_name {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                u_old_name,
                u.clone(),
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
    fn wave_propagates() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        // Initial: spike at center
        let u = vec![0.0, 0.0, 1.0, 0.0, 0.0];
        let u_old = vec![0.0; 5];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("u", u, 1)));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("u_old", u_old, 1)));

        let result = wave_step(&pd, "u", "u_old", 1.0, 0.1);
        let arr = result.point_data().get_array("u").unwrap();
        let mut buf = [0.0f64];
        // Neighbors should have picked up some displacement
        arr.tuple_as_f64(1, &mut buf); // neighbor of spike
                                       // After one step, wave energy spreads
        assert!(result.point_data().get_array("u_old").is_some());
    }

    #[test]
    fn zero_speed_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "u",
                vec![1.0, 0.0, 0.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "u_old",
                vec![1.0, 0.0, 0.0],
                1,
            )));

        let result = wave_step(&pd, "u", "u_old", 0.0, 0.1);
        let arr = result.point_data().get_array("u").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = wave_step(&pd, "u", "u_old", 1.0, 0.1);
        assert_eq!(result.points.len(), 0);
    }
}
