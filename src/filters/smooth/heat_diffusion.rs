use crate::data::{AnyDataArray, DataArray, PolyData};

/// Simulate heat diffusion on a mesh from initial temperatures.
///
/// Iteratively averages each vertex's scalar value with its neighbors,
/// weighted by `diffusivity`. The array `array_name` is used as initial
/// conditions and updated in place.
pub fn heat_diffusion(
    input: &PolyData,
    array_name: &str,
    diffusivity: f64,
    iterations: usize,
) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = input.points.len();
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (i, v) in values.iter_mut().enumerate() {
        arr.tuple_as_f64(i, &mut buf);
        *v = buf[0];
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

    let dt = diffusivity.clamp(0.0, 0.5);
    for _ in 0..iterations {
        let mut new = values.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let avg: f64 =
                neighbors[i].iter().map(|&j| values[j]).sum::<f64>() / neighbors[i].len() as f64;
            new[i] = values[i] + dt * (avg - values[i]);
        }
        values = new;
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
    fn heat_spreads() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "T",
                vec![100.0, 0.0, 0.0],
                1,
            )));

        let result = heat_diffusion(&pd, "T", 0.5, 10);
        let arr = result.point_data().get_array("T").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 100.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn equilibrium() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "T",
                vec![100.0, 0.0, 0.0],
                1,
            )));

        let result = heat_diffusion(&pd, "T", 0.5, 1000);
        let arr = result.point_data().get_array("T").unwrap();
        let mut buf = [0.0f64];
        let mut vals = Vec::new();
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            vals.push(buf[0]);
        }
        // Should converge to ~33.3
        assert!((vals[0] - vals[1]).abs() < 1.0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = heat_diffusion(&pd, "nope", 0.5, 10);
        assert_eq!(result.points.len(), 0);
    }
}
