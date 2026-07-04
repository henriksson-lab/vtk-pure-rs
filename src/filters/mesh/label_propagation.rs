//! Label propagation on meshes: spread labels from seeds via connectivity.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Propagate integer labels from seed vertices to their neighbors.
///
/// Each unlabeled vertex adopts the majority label among its labeled neighbors.
pub fn propagate_labels(mesh: &PolyData, array_name: &str, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let adj = build_adj(mesh, n);
    let mut labels: Vec<i64> = (0..n)
        .map(|i| {
            let mut buf = [0.0f64];
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as i64
        })
        .collect();

    for _ in 0..iterations {
        let mut new_labels = labels.clone();
        for i in 0..n {
            if labels[i] >= 0 {
                continue;
            } // already labeled
            if adj[i].is_empty() {
                continue;
            }
            let mut counts: std::collections::HashMap<i64, usize> =
                std::collections::HashMap::new();
            for &j in &adj[i] {
                if labels[j] >= 0 {
                    *counts.entry(labels[j]).or_insert(0) += 1;
                }
            }
            if let Some((&best_label, _)) =
                counts
                    .iter()
                    .max_by(|(label_a, count_a), (label_b, count_b)| {
                        count_a.cmp(count_b).then_with(|| label_b.cmp(label_a))
                    })
            {
                new_labels[i] = best_label;
            }
        }
        labels = new_labels;
    }

    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Random walk label propagation: each vertex randomly picks a neighbor's label.
pub fn random_walk_labels(
    mesh: &PolyData,
    array_name: &str,
    iterations: usize,
    seed: u64,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let adj = build_adj(mesh, n);
    let mut labels: Vec<i64> = (0..n)
        .map(|i| {
            let mut buf = [0.0f64];
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as i64
        })
        .collect();

    let mut rng_state = seed.wrapping_add(1);
    let next_rng = |s: &mut u64| -> usize {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*s >> 33) as usize
    };

    for _ in 0..iterations {
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let idx = next_rng(&mut rng_state) % adj[i].len();
            let nb = adj[i][idx];
            if labels[nb] >= 0 {
                labels[i] = labels[nb];
            }
        }
    }

    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    add_adj_cells(mesh.polys.iter(), true, n, &mut adj);
    add_adj_cells(mesh.lines.iter(), false, n, &mut adj);
    adj.into_iter()
        .map(|s| {
            let mut neighbors: Vec<usize> = s.into_iter().collect();
            neighbors.sort_unstable();
            neighbors
        })
        .collect()
}

fn add_adj_cells<'a, I>(
    cells: I,
    closed: bool,
    n: usize,
    adj: &mut [std::collections::HashSet<usize>],
) where
    I: IntoIterator<Item = &'a [i64]>,
{
    for cell in cells {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        let edge_count = if closed { nc } else { nc - 1 };
        for i in 0..edge_count {
            if cell[i] < 0 || cell[(i + 1) % nc] < 0 {
                continue;
            }
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn propagate() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        // Label vertex 0 as 1, others as -1
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "label",
                vec![1.0, -1.0, -1.0, -1.0],
                1,
            )));
        let result = propagate_labels(&mesh, "label", 5);
        let arr = result.point_data().get_array("label").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0); // should have propagated
    }
    #[test]
    fn random_walk() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "label",
                vec![1.0, 2.0, -1.0],
                1,
            )));
        let result = random_walk_labels(&mesh, "label", 20, 42);
        let arr = result.point_data().get_array("label").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] >= 1.0); // should have picked up a label
    }

    #[test]
    fn propagation_tie_uses_lowest_label() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.lines.push_cell(&[1, 2]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "label",
                vec![2.0, -1.0, 1.0],
                1,
            )));

        let result = propagate_labels(&mesh, "label", 1);
        let arr = result.point_data().get_array("label").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
