//! Spectral mesh clustering using Fiedler vector.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn spectral_cluster_2(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 2 {
        return mesh.clone();
    }
    let nb = build_adjacency(mesh);
    let v = fiedler_vector(&nb, iterations);
    let labels: Vec<f64> = v
        .iter()
        .map(|&x| if x >= 0.0 { 1.0 } else { 0.0 })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Cluster", labels, 1)));
    r.point_data_mut().set_active_scalars("Cluster");
    r
}
pub fn spectral_cluster_k(mesh: &PolyData, k: usize, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if k <= 1 {
        return mesh.clone();
    }
    if n < 2 {
        return mesh.clone();
    }
    let nb = build_adjacency(mesh);
    let fiedler = fiedler_vector(&nb, iterations);
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        fiedler[a]
            .partial_cmp(&fiedler[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let clusters = k.min(n);
    let mut labels = vec![0.0f64; n];
    for (rank, &point_id) in order.iter().enumerate() {
        labels[point_id] = (rank * clusters / n) as f64;
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Cluster", labels, 1)));
    result.point_data_mut().set_active_scalars("Cluster");
    result
}

fn build_adjacency(mesh: &PolyData) -> Vec<Vec<usize>> {
    let n = mesh.points.len();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_neighbor_pair(&mut nb, n, cell[i], cell[(i + 1) % nc]);
        }
    }
    nb
}

fn add_neighbor_pair(nb: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

fn fiedler_vector(nb: &[Vec<usize>], iterations: usize) -> Vec<f64> {
    let n = nb.len();
    let mut v: Vec<f64> = (0..n).map(|i| i as f64 / n as f64 - 0.5).collect();
    for _ in 0..iterations {
        let mut lv = vec![0.0f64; n];
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            lv[i] = nb[i].len() as f64 * v[i];
            for &j in &nb[i] {
                lv[i] -= v[j];
            }
        }
        let mean = lv.iter().sum::<f64>() / n as f64;
        for x in &mut lv {
            *x -= mean;
        }
        let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
        v = lv.iter().map(|x| x / norm).collect();
    }
    v
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_2() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = spectral_cluster_2(&m, 30);
        assert!(r.point_data().get_array("Cluster").is_some());
    }
    #[test]
    fn test_k() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = spectral_cluster_k(&m, 3, 30);
        let arr = r.point_data().get_array("Cluster").unwrap();
        let mut buf = [0.0];
        let mut labels: Vec<i64> = (0..arr.num_tuples())
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0] as i64
            })
            .collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), 3);
    }
    #[test]
    fn test_invalid_cell_ids_are_ignored() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.polys.push_cell(&[-1, 0, 1]);
        m.polys.push_cell(&[0, 1, 99]);
        let r = spectral_cluster_2(&m, 2);
        assert!(r.point_data().get_array("Cluster").is_some());
    }
}
