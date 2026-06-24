//! Cluster faces by normal direction using k-means on normals.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn cluster_by_normal(mesh: &PolyData, k: usize, iterations: usize) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let k = k.max(1).min(nc);
    let normals: Vec<[f64; 3]> = cells.iter().map(|c| fnorm(c, mesh)).collect();
    // Initialize centers evenly spaced
    let mut centers: Vec<[f64; 3]> = (0..k)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / k as f64;
            [a.cos(), a.sin(), 0.0]
        })
        .collect();
    let mut labels = vec![0usize; nc];
    for _ in 0..iterations {
        for ci in 0..nc {
            let n = normals[ci];
            let mut best = 0;
            let mut bd = -2.0f64;
            for (ki, c) in centers.iter().enumerate() {
                let dot = n[0] * c[0] + n[1] * c[1] + n[2] * c[2];
                if dot > bd {
                    bd = dot;
                    best = ki;
                }
            }
            labels[ci] = best;
        }
        for ki in 0..k {
            let mut sum = [0.0, 0.0, 0.0];
            let mut cnt = 0;
            for ci in 0..nc {
                if labels[ci] == ki {
                    sum[0] += normals[ci][0];
                    sum[1] += normals[ci][1];
                    sum[2] += normals[ci][2];
                    cnt += 1;
                }
            }
            if cnt > 0 {
                let l = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2])
                    .sqrt()
                    .max(1e-15);
                centers[ki] = [sum[0] / l, sum[1] / l, sum[2] / l];
            }
        }
    }
    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalCluster",
            data,
            1,
        )));
    r
}
fn fnorm(c: &[i64], m: &PolyData) -> [f64; 3] {
    if c.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = m.points.get(c[0] as usize);
    let b = m.points.get(c[1] as usize);
    let cc = m.points.get(c[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / l, n[1] / l, n[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = cluster_by_normal(&m, 2, 10);
        let arr = r.cell_data().get_array("NormalCluster").unwrap();
        let mut b1 = [0.0];
        let mut b2 = [0.0];
        arr.tuple_as_f64(0, &mut b1);
        arr.tuple_as_f64(1, &mut b2);
        assert_ne!(b1[0], b2[0]);
    } // different normals -> different clusters
}
