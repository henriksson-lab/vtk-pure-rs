use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, HashSet};

/// Tutte embedding: parameterize a disk-topology mesh to a unit circle.
///
/// Boundary vertices are placed uniformly on a circle, interior vertices
/// are placed at the average of their neighbors (convex combination).
/// Adds "TutteU" and "TutteV" scalar arrays.
pub fn tutte_parameterize(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for cell in input.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
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

    // Find boundary loop
    let mut boundary_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }
    for neighbors in boundary_adj.values_mut() {
        neighbors.sort_unstable();
    }

    // Trace boundary loop
    let mut boundary_loop = Vec::new();
    if let Some(&start) = boundary_adj.keys().min() {
        let mut visited = HashSet::new();
        let mut cur = start;
        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            boundary_loop.push(cur);
            let next = boundary_adj
                .get(&cur)
                .and_then(|v| v.iter().find(|&&n| !visited.contains(&n)));
            match next {
                Some(&n) => cur = n,
                None => break,
            }
        }
    }

    if boundary_loop.len() < 3 {
        return input.clone();
    }

    let boundary_set: HashSet<usize> = boundary_loop.iter().copied().collect();
    let nb = boundary_loop.len();

    // Place boundary on unit circle
    let mut u = vec![0.0f64; n];
    let mut v = vec![0.0f64; n];
    for (i, &bi) in boundary_loop.iter().enumerate() {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / nb as f64;
        u[bi] = 0.5 + 0.5 * angle.cos();
        v[bi] = 0.5 + 0.5 * angle.sin();
    }

    // Iteratively solve for interior
    for _ in 0..200 {
        let mut next_u = u.clone();
        let mut next_v = v.clone();
        for i in 0..n {
            if boundary_set.contains(&i) || neighbors[i].is_empty() {
                continue;
            }
            let cnt = neighbors[i].len() as f64;
            let su: f64 = neighbors[i].iter().map(|&j| u[j]).sum();
            let sv: f64 = neighbors[i].iter().map(|&j| v[j]).sum();
            next_u[i] = su / cnt;
            next_v[i] = sv / cnt;
        }
        u = next_u;
        v = next_v;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TutteU", u, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TutteV", v, 1)));
    pd
}

fn valid_point_id(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutte_grid() {
        let mut pd = PolyData::new();
        for j in 0..4 {
            for i in 0..4 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..3 {
            for i in 0..3 {
                let a = (j * 4 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 5]);
                pd.polys.push_cell(&[a, a + 5, a + 4]);
            }
        }

        let result = tutte_parameterize(&pd);
        assert!(result.point_data().get_array("TutteU").is_some());
        assert!(result.point_data().get_array("TutteV").is_some());

        // UV should be in [0,1]
        let ua = result.point_data().get_array("TutteU").unwrap();
        let mut buf = [0.0f64];
        for i in 0..16 {
            ua.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0 - 0.01 && buf[0] <= 1.01);
        }
    }

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = tutte_parameterize(&pd);
        assert!(result.point_data().get_array("TutteU").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = tutte_parameterize(&pd);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn malformed_cells_do_not_panic() {
        let mut pd = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        pd.polys.push_cell(&[0, -1, 99]);

        let result = tutte_parameterize(&pd);
        assert_eq!(result.points.len(), 3);
    }
}
