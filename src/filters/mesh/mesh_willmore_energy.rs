//! Compute Willmore energy (integral of mean curvature squared).
use crate::data::PolyData;
pub fn willmore_energy(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n == 0 {
        return 0.0;
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
    let nm = calc_nm(mesh);
    let mut total = 0.0;
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let p = mesh.points.get(i);
        let ni = nm[i];
        let k = nb[i].len() as f64;
        let mut lap = [0.0, 0.0, 0.0];
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            lap[0] += q[0] - p[0];
            lap[1] += q[1] - p[1];
            lap[2] += q[2] - p[2];
        }
        let hn = (lap[0] * ni[0] + lap[1] * ni[1] + lap[2] * ni[2]) / k;
        total += hn * hn;
    }
    total
}
pub fn normalized_willmore(mesh: &PolyData) -> f64 {
    let w = willmore_energy(mesh);
    let n = mesh.points.len();
    if n > 0 {
        w / n as f64
    } else {
        0.0
    }
}
pub fn is_willmore_surface(mesh: &PolyData, tolerance: f64) -> bool {
    // A Willmore surface has W ≈ 4π (for a sphere)
    let w = willmore_energy(mesh);
    (w - 4.0 * std::f64::consts::PI).abs() < tolerance
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
    fn test_energy() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let w = willmore_energy(&m);
        assert!(w > 0.0);
    }
    #[test]
    fn test_normalized() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let nw = normalized_willmore(&m);
        assert!(nw >= 0.0);
    }
}
