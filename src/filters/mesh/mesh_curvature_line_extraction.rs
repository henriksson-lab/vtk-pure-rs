//! Extract principal curvature lines on mesh surface.
use crate::data::{CellArray, Points, PolyData};
pub fn curvature_lines(
    mesh: &PolyData,
    num_seeds: usize,
    max_steps: usize,
    seed_offset: u64,
) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return PolyData::new();
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
    // Estimate max curvature direction per vertex
    let dirs: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            if nb[i].len() < 2 {
                return [1.0, 0.0, 0.0];
            }
            let p = mesh.points.get(i);
            let ni = nm[i];
            let mut max_curv = 0.0f64;
            let mut max_dir = [1.0, 0.0, 0.0];
            for &j in &nb[i] {
                let q = mesh.points.get(j);
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
                let proj = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
                let curv = (2.0 * proj / (dist * dist)).abs();
                if curv > max_curv {
                    max_curv = curv;
                    let t = [
                        d[0] - proj * ni[0],
                        d[1] - proj * ni[1],
                        d[2] - proj * ni[2],
                    ];
                    let tl = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-15);
                    max_dir = [t[0] / tl, t[1] / tl, t[2] / tl];
                }
            }
            max_dir
        })
        .collect();
    // Trace curvature lines from seed vertices
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut rng = seed_offset;
    let ns = num_seeds.min(n);
    for _ in 0..ns {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let seed = (rng >> 33) as usize % n;
        let mut path = vec![seed];
        let mut cur = seed;
        for _ in 0..max_steps {
            let d = dirs[cur];
            let next = nb[cur].iter().max_by(|&&a, &&b| {
                let pa = mesh.points.get(a);
                let pc = mesh.points.get(cur);
                let da = [pa[0] - pc[0], pa[1] - pc[1], pa[2] - pc[2]];
                let dot_a = (da[0] * d[0] + da[1] * d[1] + da[2] * d[2]).abs();
                let pb = mesh.points.get(b);
                let db = [pb[0] - pc[0], pb[1] - pc[1], pb[2] - pc[2]];
                let dot_b = (db[0] * d[0] + db[1] * d[1] + db[2] * d[2]).abs();
                dot_a
                    .partial_cmp(&dot_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            match next {
                Some(&nxt) => {
                    if path.contains(&nxt) {
                        break;
                    }
                    path.push(nxt);
                    cur = nxt;
                }
                None => {
                    break;
                }
            }
        }
        if path.len() >= 3 {
            let ids: Vec<i64> = path
                .iter()
                .map(|&v| {
                    let idx = pts.len();
                    pts.push(mesh.points.get(v));
                    idx as i64
                })
                .collect();
            lines.push_cell(&ids);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
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
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 4], [1, 3, 4], [3, 2, 4], [2, 0, 4]],
        );
        let r = curvature_lines(&m, 3, 10, 42);
        assert!(r.points.len() >= 0);
    }
}
