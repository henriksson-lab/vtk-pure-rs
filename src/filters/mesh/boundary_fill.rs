use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, HashSet, VecDeque};

/// Flood-fill a scalar value from boundary inward.
///
/// Sets boundary vertices to `value`, then propagates inward using
/// BFS with exponential decay: each hop multiplies by `decay_factor`.
/// Adds "BoundaryFill" scalar array.
pub fn boundary_fill(input: &PolyData, value: f64, decay_factor: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    // Find boundary vertices
    let mut boundary: HashSet<usize> = HashSet::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }

    let mut fill = vec![0.0f64; n];
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();

    for &v in &boundary {
        fill[v] = value;
        visited[v] = true;
        queue.push_back((v, value));
    }

    while let Some((v, val)) = queue.pop_front() {
        let next_val = val * decay_factor;
        if next_val.abs() < 1e-10 {
            continue;
        }
        for &nb in &neighbors[v] {
            if !visited[nb] {
                visited[nb] = true;
                fill[nb] = next_val;
                queue.push_back((nb, next_val));
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryFill",
            fill,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_from_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }

        let result = boundary_fill(&pd, 1.0, 0.5);
        let arr = result.point_data().get_array("BoundaryFill").unwrap();
        let mut buf = [0.0f64];
        // Boundary vertices get full value
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        // Center gets decayed value
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1.0);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn closed_mesh_no_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = boundary_fill(&pd, 1.0, 0.5);
        let arr = result.point_data().get_array("BoundaryFill").unwrap();
        let mut buf = [0.0f64];
        // No boundary -> all zeros
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = boundary_fill(&pd, 1.0, 0.5);
        assert_eq!(result.points.len(), 0);
    }
}
