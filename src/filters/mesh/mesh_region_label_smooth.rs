//! Smooth region labels using mode filtering.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn smooth_labels(mesh: &PolyData, label_array: &str, iterations: usize) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let nc = mesh.polys.num_cells();
    let mut buf = [0.0f64];
    let mut labels: Vec<usize> = (0..nc)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as usize
        })
        .collect();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let n = c.len();
        for i in 0..n {
            let a = c[i] as usize;
            let b = c[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for (_, faces) in &ef {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }
    for _ in 0..iterations {
        let prev = labels.clone();
        for ci in 0..nc {
            let mut counts: std::collections::HashMap<usize, usize> =
                std::collections::HashMap::new();
            *counts.entry(prev[ci]).or_insert(0) += 2; // bias toward self
            for &ni in &adj[ci] {
                *counts.entry(prev[ni]).or_insert(0) += 1;
            }
            labels[ci] = *counts.iter().max_by_key(|(_, &c)| c).unwrap().0;
        }
    }
    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
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
        m.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "L",
                vec![1.0, 1.0],
                1,
            )));
        let r = smooth_labels(&m, "L", 3);
        assert!(r.cell_data().get_array("L").is_some());
    }
}
