//! Merge small watershed regions into neighbors.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn merge_small_regions(mesh: &PolyData, label_array: &str, min_vertices: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(label_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let mut labels: Vec<usize> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as usize
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    // Count region sizes
    let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &l in &labels {
        *counts.entry(l).or_insert(0) += 1;
    }
    // Merge small regions into largest neighbor region
    let mut changed = true;
    while changed {
        changed = false;
        let small: Vec<usize> = counts
            .iter()
            .filter(|(_, &c)| c < min_vertices)
            .map(|(&l, _)| l)
            .collect();
        for sl in small {
            let verts: Vec<usize> = (0..n).filter(|&i| labels[i] == sl).collect();
            let mut neighbor_counts: std::collections::HashMap<usize, usize> =
                std::collections::HashMap::new();
            for &vi in &verts {
                for &ni in &nb[vi] {
                    if labels[ni] != sl {
                        *neighbor_counts.entry(labels[ni]).or_insert(0) += 1;
                    }
                }
            }
            if let Some((&best, _)) = neighbor_counts.iter().max_by_key(|(_, &c)| c) {
                for &vi in &verts {
                    labels[vi] = best;
                }
                *counts.entry(best).or_insert(0) += verts.len();
                counts.remove(&sl);
                changed = true;
                break;
            }
        }
    }
    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(label_array, data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "W",
                vec![0.0, 1.0, 0.0, 1.0],
                1,
            )));
        let r = merge_small_regions(&m, "W", 3);
        assert!(r.point_data().get_array("W").is_some());
    }
}
