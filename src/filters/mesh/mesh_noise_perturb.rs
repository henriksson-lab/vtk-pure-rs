//! Add random noise to vertex positions.
use crate::data::PolyData;
pub fn perturb_vertices(mesh: &PolyData, amplitude: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let mut rng = seed;
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let dx = rnd(&mut rng) * amplitude;
        let dy = rnd(&mut rng) * amplitude;
        let dz = rnd(&mut rng) * amplitude;
        r.points.set(i, [p[0] + dx, p[1] + dy, p[2] + dz]);
    }
    r
}
pub fn perturb_along_normals(mesh: &PolyData, amplitude: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_normals(mesh);
    let mut rng = seed;
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let d = rnd(&mut rng) * amplitude;
        r.points.set(
            i,
            [
                p[0] + d * nm[i][0],
                p[1] + d * nm[i][1],
                p[2] + d * nm[i][2],
            ],
        );
    }
    r
}
fn rnd(s: &mut u64) -> f64 {
    *s = s
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*s >> 33) as f64 / ((1u64 << 31) - 1) as f64) * 2.0 - 1.0
}
fn calc_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(ia) = valid_point_id(cell[0], n) else {
            continue;
        };
        let a = mesh.points.get(ia);
        for i in 1..cell.len() - 1 {
            let Some(ib) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(ic) = valid_point_id(cell[i + 1], n) else {
                continue;
            };
            let b = mesh.points.get(ib);
            let c = mesh.points.get(ic);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for v in [cell[0], cell[i], cell[i + 1]] {
                if let Some(vi) = valid_point_id(v, n) {
                    nm[vi][0] += fn_[0];
                    nm[vi][1] += fn_[1];
                    nm[vi][2] += fn_[2];
                }
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
    fn test_perturb() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = perturb_vertices(&m, 0.01, 42);
        let p = r.points.get(0);
        assert!(p[0].abs() < 0.02);
    }
    #[test]
    fn test_rnd_spans_both_sides() {
        let mut s = 42;
        let mut saw_negative = false;
        let mut saw_positive = false;
        for _ in 0..100 {
            let v = rnd(&mut s);
            saw_negative |= v < 0.0;
            saw_positive |= v > 0.0;
        }
        assert!(saw_negative && saw_positive);
    }
    #[test]
    fn test_normal_perturb() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = perturb_along_normals(&m, 0.1, 42);
        assert_eq!(r.points.len(), 3);
    }
}
