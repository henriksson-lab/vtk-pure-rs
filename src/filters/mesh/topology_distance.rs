use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::VecDeque;

/// Compute topological distance (hop count) from source vertices.
///
/// Each vertex gets the minimum number of edge hops to reach any source.
/// Adds "TopologicalDistance" scalar (integer valued).
pub fn topological_distance(input: &PolyData, sources: &[usize]) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    let mut dist = vec![-1.0f64; n];
    let mut queue = VecDeque::new();
    for &s in sources {
        if s < n {
            dist[s] = 0.0;
            queue.push_back(s);
        }
    }

    while let Some(v) = queue.pop_front() {
        let d = dist[v];
        for &nb in &adj[v] {
            if dist[nb] < 0.0 {
                dist[nb] = d + 1.0;
                queue.push_back(nb);
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TopologicalDistance",
            dist,
            1,
        )));
    pd
}

/// Compute the eccentricity of each vertex (max hop distance to any other vertex).
///
/// Uses BFS from each vertex. Adds "Eccentricity" scalar.
/// Only practical for small meshes due to O(V*(V+E)) complexity.
pub fn eccentricity(input: &PolyData, max_vertices: usize) -> PolyData {
    let n = input.points.len().min(max_vertices);
    if n == 0 {
        return input.clone();
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); input.points.len()];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    let total = input.points.len();
    let mut ecc = vec![0.0f64; total];

    for src in 0..n {
        let mut dist = vec![usize::MAX; total];
        dist[src] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(src);
        let mut max_d = 0;
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if dist[nb] == usize::MAX {
                    dist[nb] = dist[v] + 1;
                    max_d = max_d.max(dist[nb]);
                    queue.push_back(nb);
                }
            }
        }
        ecc[src] = max_d as f64;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Eccentricity",
            ecc,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hop_distance() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = topological_distance(&pd, &[0]);
        let arr = result
            .point_data()
            .get_array("TopologicalDistance")
            .unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn eccentricity_line() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = eccentricity(&pd, 100);
        let arr = result.point_data().get_array("Eccentricity").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf); // endpoint has max eccentricity
        let e0 = buf[0];
        arr.tuple_as_f64(2, &mut buf); // center has min eccentricity
        let e2 = buf[0];
        assert!(e0 >= e2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = topological_distance(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
