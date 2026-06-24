//! Willmore flow (gradient descent on Willmore energy).
use crate::data::PolyData;
pub fn willmore_flow(mesh: &PolyData, steps: usize, dt: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
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
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..steps {
        let prev = pos.clone();
        // Compute normals
        let mut nm = vec![[0.0f64; 3]; n];
        for cell in mesh.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let a = prev[cell[0] as usize];
            let b = prev[cell[1] as usize];
            let c = prev[cell[2] as usize];
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
        // Compute mean curvature H
        let mut h_vec = vec![[0.0f64; 3]; n];
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                lap[0] += prev[j][0] - prev[i][0];
                lap[1] += prev[j][1] - prev[i][1];
                lap[2] += prev[j][2] - prev[i][2];
            }
            h_vec[i] = [lap[0] / k, lap[1] / k, lap[2] / k];
        }
        // Laplacian of mean curvature (biharmonic)
        let mut bilap = vec![[0.0f64; 3]; n];
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            for &j in &nb[i] {
                bilap[i][0] += h_vec[j][0] - h_vec[i][0];
                bilap[i][1] += h_vec[j][1] - h_vec[i][1];
                bilap[i][2] += h_vec[j][2] - h_vec[i][2];
            }
            bilap[i][0] /= k;
            bilap[i][1] /= k;
            bilap[i][2] /= k;
        }
        // Willmore gradient: move along -Lap(H)*n direction
        for i in 0..n {
            let hn = bilap[i][0] * nm[i][0] + bilap[i][1] * nm[i][1] + bilap[i][2] * nm[i][2];
            pos[i][0] = prev[i][0] - dt * hn * nm[i][0];
            pos[i][1] = prev[i][1] - dt * hn * nm[i][1];
            pos[i][2] = prev[i][2] - dt * hn * nm[i][2];
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = willmore_flow(&m, 3, 0.01);
        assert_eq!(r.points.len(), 4);
    }
}
