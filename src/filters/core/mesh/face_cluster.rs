use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Cluster faces by normal similarity using region growing.
///
/// Seeds are spread across the mesh and grow to neighbors with similar
/// normals (within `angle_threshold_deg`). Adds "FaceCluster" cell data.
pub fn face_cluster_by_normal(input: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let cos_thresh = angle_threshold_deg.to_radians().cos();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return input.clone();
    }

    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| compute_polygon_normal(input, c))
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }

    let mut labels = vec![0usize; nc];
    let mut current = 0;

    for seed in 0..nc {
        if labels[seed] != 0 {
            continue;
        }
        current += 1;
        let mut queue = std::collections::VecDeque::new();
        labels[seed] = current;
        queue.push_back(seed);

        while let Some(fi) = queue.pop_front() {
            for &ni in &adj[fi] {
                if labels[ni] != 0 {
                    continue;
                }
                let dot = normals[fi][0] * normals[ni][0]
                    + normals[fi][1] * normals[ni][1]
                    + normals[fi][2] * normals[ni][2];
                if dot >= cos_thresh {
                    labels[ni] = current;
                    queue.push_back(ni);
                }
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceCluster",
            labels_f,
            1,
        )));
    pd
}

fn compute_polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0; 3];
    }

    let mut n = [0.0; 3];
    let mut prev = input.points.get(cell[cell.len() - 1] as usize);
    for &pid in cell {
        let cur = input.points.get(pid as usize);
        n[0] += (prev[1] - cur[1]) * (prev[2] + cur[2]);
        n[1] += (prev[2] - cur[2]) * (prev[0] + cur[0]);
        n[2] += (prev[0] - cur[0]) * (prev[1] + cur[1]);
        prev = cur;
    }

    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0; 3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_one_cluster() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = face_cluster_by_normal(&pd, 10.0);
        let arr = result.cell_data().get_array("FaceCluster").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let c1 = buf[0];
        assert_eq!(c0, c1); // same cluster
    }

    #[test]
    fn perpendicular_two_clusters() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]); // XY plane
        pd.points.push([0.5, 0.0, 1.0]); // XZ plane
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = face_cluster_by_normal(&pd, 30.0);
        let arr = result.cell_data().get_array("FaceCluster").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let c1 = buf[0];
        assert_ne!(c0, c1); // different clusters
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = face_cluster_by_normal(&pd, 10.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
