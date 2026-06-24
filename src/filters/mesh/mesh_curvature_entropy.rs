//! Compute curvature entropy as a shape complexity measure.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn curvature_entropy(mesh: &PolyData, num_bins: usize) -> f64 {
    let n = mesh.points.len();
    if n < 3 {
        return 0.0;
    }
    let nb = num_bins.max(2);
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) {
                    neighbors[a].push(b);
                }
                if !neighbors[b].contains(&a) {
                    neighbors[b].push(a);
                }
            }
        }
    }
    let nm = calc_nm(mesh);
    let curvatures: Vec<f64> = (0..n)
        .map(|i| {
            if neighbors[i].is_empty() {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let ni = nm[i];
            let k = neighbors[i].len() as f64;
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &neighbors[i] {
                let q = mesh.points.get(j);
                lap[0] += q[0] - p[0];
                lap[1] += q[1] - p[1];
                lap[2] += q[2] - p[2];
            }
            (lap[0] * ni[0] + lap[1] * ni[1] + lap[2] * ni[2]) / k
        })
        .collect();
    let mn = curvatures.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = curvatures.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let mut hist = vec![0usize; nb];
    for &c in &curvatures {
        let bi = (((c - mn) / range * nb as f64).floor() as usize).min(nb - 1);
        hist[bi] += 1;
    }
    let nf = n as f64;
    hist.iter()
        .filter(|&&h| h > 0)
        .map(|&h| {
            let p = h as f64 / nf;
            -p * p.ln()
        })
        .sum()
}
pub fn attach_curvature_entropy(mesh: &PolyData, ring_size: usize) -> PolyData {
    let n = mesh.points.len();
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) {
                    neighbors[a].push(b);
                }
                if !neighbors[b].contains(&a) {
                    neighbors[b].push(a);
                }
            }
        }
    }
    let nm = calc_nm(mesh);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if neighbors[i].is_empty() {
                return 0.0;
            }
            // Local entropy from curvature variation in ring
            let mut ring = vec![i];
            let mut frontier = vec![i];
            let mut visited = vec![false; n];
            visited[i] = true;
            for _ in 0..ring_size {
                let mut next = Vec::new();
                for &v in &frontier {
                    for &u in &neighbors[v] {
                        if !visited[u] {
                            visited[u] = true;
                            next.push(u);
                            ring.push(u);
                        }
                    }
                }
                frontier = next;
            }
            let curvs: Vec<f64> = ring
                .iter()
                .map(|&vi| {
                    let p = mesh.points.get(vi);
                    let ni = nm[vi];
                    let k = neighbors[vi].len().max(1) as f64;
                    let mut lap = [0.0, 0.0, 0.0];
                    for &j in &neighbors[vi] {
                        let q = mesh.points.get(j);
                        lap[0] += q[0] - p[0];
                        lap[1] += q[1] - p[1];
                        lap[2] += q[2] - p[2];
                    }
                    (lap[0] * ni[0] + lap[1] * ni[1] + lap[2] * ni[2]) / k
                })
                .collect();
            let mean = curvs.iter().sum::<f64>() / curvs.len() as f64;
            let var = curvs.iter().map(|&c| (c - mean).powi(2)).sum::<f64>() / curvs.len() as f64;
            var.sqrt() // standard deviation as local complexity
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvEntropy",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("CurvEntropy");
    r
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_global() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let e = curvature_entropy(&m, 10);
        assert!(e > 0.0);
    }
    #[test]
    fn test_local() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = attach_curvature_entropy(&m, 1);
        assert!(r.point_data().get_array("CurvEntropy").is_some());
    }
}
