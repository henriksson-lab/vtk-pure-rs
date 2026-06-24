//! Morse complex-based mesh segmentation.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn morse_segmentation(mesh: &PolyData, array_name: &str, merge_threshold: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
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
    // Descending manifold: flow to minimum
    let mut basin = vec![0usize; n];
    for i in 0..n {
        let mut cur = i;
        let mut steps = 0;
        while steps < n {
            let next = nb[cur]
                .iter()
                .filter(|&&j| vals[j] < vals[cur])
                .min_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            match next {
                Some(&nxt) => {
                    cur = nxt;
                    steps += 1;
                }
                None => {
                    break;
                }
            }
        }
        basin[i] = cur;
    }
    // Merge small basins
    let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &b in &basin {
        *counts.entry(b).or_insert(0) += 1;
    }
    let min_size = (n as f64 * merge_threshold) as usize;
    let mut parent: Vec<usize> = (0..n).collect();
    for (&root, &count) in &counts {
        if count < min_size {
            // Find nearest larger basin
            let verts: Vec<usize> = (0..n).filter(|&i| basin[i] == root).collect();
            let mut best_neighbor = root;
            let mut best_d = f64::INFINITY;
            for &vi in &verts {
                for &ni in &nb[vi] {
                    if basin[ni] != root {
                        let d = (vals[vi] - vals[ni]).abs();
                        if d < best_d {
                            best_d = d;
                            best_neighbor = basin[ni];
                        }
                    }
                }
            }
            if best_neighbor != root {
                for &vi in &verts {
                    basin[vi] = best_neighbor;
                }
            }
        }
    }
    // Relabel
    let mut label_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut next = 0;
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            *label_map.entry(basin[i]).or_insert_with(|| {
                let l = next;
                next += 1;
                l
            }) as f64
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorseSegment",
            labels,
            1,
        )));
    r.point_data_mut().set_active_scalars("MorseSegment");
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
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 3.0, 0.5],
                1,
            )));
        let r = morse_segmentation(&m, "h", 0.1);
        assert!(r.point_data().get_array("MorseSegment").is_some());
    }
}
