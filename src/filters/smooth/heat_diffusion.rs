use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

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
    if iterations == 0 || n == 0 || arr.num_tuples() != n {
        return input.clone();
    }

    let num_comp = arr.num_components();
    let mut values = vec![0.0f64; n * num_comp];
    let mut buf = vec![0.0f64; num_comp];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        for c in 0..num_comp {
            values[i * num_comp + c] = buf[c];
        }
    }

    let neighbors = build_adjacency(input, n);

    let dt = diffusivity.clamp(0.0, 0.5);
    for _ in 0..iterations {
        let mut new = values.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            for c in 0..num_comp {
                let avg: f64 = neighbors[i]
                    .iter()
                    .map(|&j| values[j * num_comp + c])
                    .sum::<f64>()
                    / neighbors[i].len() as f64;
                let idx = i * num_comp + c;
                new[idx] = values[idx] + dt * (avg - values[idx]);
            }
        }
        values = new;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, num_comp,
        )));
    pd
}

fn build_adjacency(input: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        add_closed_cell_edges(cell, n, &mut neighbors);
    }
    for cell in input.lines.iter() {
        add_open_cell_edges(cell, n, &mut neighbors);
    }
    neighbors
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect()
}

fn add_closed_cell_edges(cell: &[i64], n: usize, neighbors: &mut [HashSet<usize>]) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        add_edge(cell[i], cell[(i + 1) % cell.len()], n, neighbors);
    }
}

fn add_open_cell_edges(cell: &[i64], n: usize, neighbors: &mut [HashSet<usize>]) {
    for edge in cell.windows(2) {
        add_edge(edge[0], edge[1], n, neighbors);
    }
}

fn add_edge(a: i64, b: i64, n: usize, neighbors: &mut [HashSet<usize>]) {
    let (Some(a), Some(b)) = (valid_point_id(a, n), valid_point_id(b, n)) else {
        return;
    };
    if a != b {
        neighbors[a].insert(b);
        neighbors[b].insert(a);
    }
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
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
