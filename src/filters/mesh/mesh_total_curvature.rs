//! Compute total Gaussian and mean curvature integrals (Gauss-Bonnet).
use crate::data::PolyData;
pub fn total_gaussian_curvature(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    let mut angle_sum = vec![0.0f64; n];
    let mut incident = vec![false; n];
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            let vi_id = cell[i];
            let prev_id = cell[(i + nc - 1) % nc];
            let next_id = cell[(i + 1) % nc];
            if let Some((a, b)) = valid_edge(vi_id, next_id, n) {
                *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
            if vi_id < 0 || prev_id < 0 || next_id < 0 {
                continue;
            }
            let vi = vi_id as usize;
            let prev = prev_id as usize;
            let next = next_id as usize;
            if vi >= n || prev >= n || next >= n {
                continue;
            }
            incident[vi] = true;
            let p = mesh.points.get(vi);
            let a = mesh.points.get(prev);
            let b = mesh.points.get(next);
            let va = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
            let vb = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
            let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
            let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
            if la > 1e-15 && lb > 1e-15 {
                angle_sum[vi] += ((va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]) / (la * lb))
                    .clamp(-1.0, 1.0)
                    .acos();
            }
        }
    }
    let mut is_boundary = vec![false; n];
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            is_boundary[a] = true;
            is_boundary[b] = true;
        }
    }
    angle_sum
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            if !incident[i] {
                0.0
            } else if is_boundary[i] {
                std::f64::consts::PI - s
            } else {
                2.0 * std::f64::consts::PI - s
            }
        })
        .sum()
}
pub fn euler_characteristic_from_curvature(mesh: &PolyData) -> f64 {
    total_gaussian_curvature(mesh) / (2.0 * std::f64::consts::PI)
}
pub fn total_mean_curvature(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a_id = cell[i];
            let b_id = cell[(i + 1) % nc];
            if a_id < 0 || b_id < 0 {
                continue;
            }
            let a = a_id as usize;
            let b = b_id as usize;
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
        total += (lap[0] * ni[0] + lap[1] * ni[1] + lap[2] * ni[2]) / k;
    }
    total
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if cell[0] < 0 || cell[1] < 0 || cell[2] < 0 {
            continue;
        }
        let ia = cell[0] as usize;
        let ib = cell[1] as usize;
        let ic = cell[2] as usize;
        if ia >= n || ib >= n || ic >= n {
            continue;
        }
        let a = mesh.points.get(ia);
        let b = mesh.points.get(ib);
        let c = mesh.points.get(ic);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            if v >= 0 && (v as usize) < n {
                let vi = v as usize;
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
fn valid_edge(a: i64, b: i64, number_of_points: usize) -> Option<(usize, usize)> {
    if a >= 0 && b >= 0 {
        let a = a as usize;
        let b = b as usize;
        if a < number_of_points && b < number_of_points {
            return Some((a, b));
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gauss_bonnet() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let chi = euler_characteristic_from_curvature(&m);
        assert!((chi - 2.0).abs() < 0.5);
    } // closed -> chi=2
    #[test]
    fn test_mean() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let tm = total_mean_curvature(&m);
        assert!(tm.is_finite());
    }
    #[test]
    fn test_open_disk_curvature_uses_boundary_defect() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let chi = euler_characteristic_from_curvature(&m);
        assert!((chi - 1.0).abs() < 1e-12);
    }
}
