//! Cluster mesh faces by their normal direction (k-means on normals).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn normal_cluster(mesh: &PolyData, k: usize) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        let mut result = mesh.clone();
        result
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "NormalCluster",
                Vec::new(),
                1,
            )));
        return result;
    }
    let normals: Vec<[f64; 3]> = cells.iter().map(|cell| face_normal(mesh, cell)).collect();
    let k = k.max(1).min(nc);
    // Initialize centroids from evenly spaced faces
    let mut centroids: Vec<[f64; 3]> = (0..k).map(|i| normals[i * nc / k]).collect();
    let mut labels = vec![0usize; nc];
    for _ in 0..20 {
        // Assign
        for (fi, nn) in normals.iter().enumerate() {
            let mut best = 0;
            let mut best_dot = f64::NEG_INFINITY;
            for (ci, cn) in centroids.iter().enumerate() {
                let dot = nn[0] * cn[0] + nn[1] * cn[1] + nn[2] * cn[2];
                if dot > best_dot {
                    best_dot = dot;
                    best = ci;
                }
            }
            labels[fi] = best;
        }
        // Update centroids
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];
        for (fi, &l) in labels.iter().enumerate() {
            sums[l][0] += normals[fi][0];
            sums[l][1] += normals[fi][1];
            sums[l][2] += normals[fi][2];
            counts[l] += 1;
        }
        for ci in 0..k {
            if counts[ci] > 0 {
                let c = counts[ci] as f64;
                let nn = [sums[ci][0] / c, sums[ci][1] / c, sums[ci][2] / c];
                let len = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
                if len > 1e-15 {
                    centroids[ci] = [nn[0] / len, nn[1] / len, nn[2] / len];
                }
            }
        }
    }
    let label_data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalCluster",
            label_data,
            1,
        )));
    result
}

fn face_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let Some(a) = valid_point_id(cell[0], mesh.points.len()) else {
        return [0.0, 0.0, 1.0];
    };
    let Some(b) = valid_point_id(cell[1], mesh.points.len()) else {
        return [0.0, 0.0, 1.0];
    };
    let Some(c) = valid_point_id(cell[2], mesh.points.len()) else {
        return [0.0, 0.0, 1.0];
    };
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let pc = mesh.points.get(c);
    let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
    let nx = u[1] * v[2] - u[2] * v[1];
    let ny = u[2] * v[0] - u[0] * v[2];
    let nz = u[0] * v[1] - u[1] * v[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-15 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
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
    fn test_normal_cluster() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let r = normal_cluster(&mesh, 2);
        assert!(r.cell_data().get_array("NormalCluster").is_some());
    }

    #[test]
    fn empty_input_adds_empty_cluster_array() {
        let mesh = PolyData::new();
        let r = normal_cluster(&mesh, 2);
        let arr = r.cell_data().get_array("NormalCluster").unwrap();
        assert_eq!(arr.num_tuples(), 0);
    }
}
