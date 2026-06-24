use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, HashSet};

/// Compute harmonic coordinates on a mesh with boundary conditions.
///
/// Solves Laplace's equation ∇²f = 0 on the mesh graph with fixed
/// values at boundary vertices. Uses iterative relaxation (Gauss-Seidel).
/// Adds the result as a named scalar array.
pub fn harmonic_solve(
    input: &PolyData,
    boundary_values: &[(usize, f64)],
    output_name: &str,
    iterations: usize,
) -> PolyData {
    let n = input.points.len();
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

    let fixed: HashMap<usize, f64> = boundary_values
        .iter()
        .filter(|&&(i, _)| i < n)
        .cloned()
        .collect();
    let mut values = vec![0.0f64; n];
    for (&i, &v) in &fixed {
        values[i] = v;
    }

    for _ in 0..iterations {
        for i in 0..n {
            if fixed.contains_key(&i) {
                continue;
            }
            if neighbors[i].is_empty() {
                continue;
            }
            let avg: f64 =
                neighbors[i].iter().map(|&j| values[j]).sum::<f64>() / neighbors[i].len() as f64;
            values[i] = avg;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output_name,
            values,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_linearly() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = harmonic_solve(&pd, &[(0, 0.0), (4, 1.0)], "h", 100);
        let arr = result.point_data().get_array("h").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-5);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-5);
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] > 0.2 && buf[0] < 0.8);
    }

    #[test]
    fn boundary_preserved() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = harmonic_solve(&pd, &[(0, 42.0)], "f", 50);
        let arr = result.point_data().get_array("f").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 42.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = harmonic_solve(&pd, &[], "f", 10);
        assert_eq!(result.points.len(), 0);
    }
}
