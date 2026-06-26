use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, HashSet, VecDeque};

/// Compute the distance from each vertex to the nearest boundary edge.
///
/// Uses BFS from boundary vertices, counting edge hops.
/// Adds "BoundaryDistance" scalar array (integer hop count).
pub fn boundary_distance(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Find boundary edges
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            let key = if (a as i64) < (b as i64) {
                (a as i64, b as i64)
            } else {
                (b as i64, a as i64)
            };
            *edge_count.entry(key).or_insert(0) += 1;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    // Identify boundary vertices
    let mut boundary: HashSet<usize> = HashSet::new();
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary.insert(a as usize);
            boundary.insert(b as usize);
        }
    }

    // BFS from boundary
    let mut dist = vec![-1.0f64; n];
    let mut queue = VecDeque::new();
    for &v in &boundary {
        dist[v] = 0.0;
        queue.push_back(v);
    }

    while let Some(v) = queue.pop_front() {
        let d = dist[v];
        for &nb in &neighbors[v] {
            if dist[nb] < 0.0 {
                dist[nb] = d + 1.0;
                queue.push_back(nb);
            }
        }
    }

    // Vertices not reachable (isolated) get 0
    for d in &mut dist {
        if *d < 0.0 {
            *d = 0.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryDistance",
            dist,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_zero() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = boundary_distance(&pd);
        let arr = result.point_data().get_array("BoundaryDistance").unwrap();
        let mut buf = [0.0f64];
        // All vertices are boundary for a single triangle
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn interior_positive() {
        let mut pd = PolyData::new();
        // Center vertex surrounded by triangles
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }

        let result = boundary_distance(&pd);
        let arr = result.point_data().get_array("BoundaryDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0); // center is interior
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = boundary_distance(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
