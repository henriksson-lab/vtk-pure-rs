//! Adaptive edge collapse driven by curvature (preserve detail where curved).
use crate::data::{CellArray, Points, PolyData};
pub fn adaptive_edge_collapse(
    mesh: &PolyData,
    target_faces: usize,
    curvature_weight: f64,
) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let npts = pts.len();
    let mut remap: Vec<usize> = (0..npts).collect();
    let mut removed = vec![false; tris.len()];
    // Estimate curvature
    let curv = estimate_curv(&pts, &tris, npts);

    while tris.len() - removed.iter().filter(|&&r| r).count() > target_faces {
        let mut best_cost = f64::INFINITY;
        let mut best_a = 0;
        let mut best_b = 0;
        for (ti, t) in tris.iter().enumerate() {
            if removed[ti] {
                continue;
            }
            let v = [res(t[0], &remap), res(t[1], &remap), res(t[2], &remap)];
            for i in 0..3 {
                let a = v[i];
                let b = v[(i + 1) % 3];
                if a == b {
                    continue;
                }
                let edge_len = (pts[a][0] - pts[b][0]).powi(2)
                    + (pts[a][1] - pts[b][1]).powi(2)
                    + (pts[a][2] - pts[b][2]).powi(2);
                let curv_penalty = if a < curv.len() && b < curv.len() {
                    (curv[a] + curv[b]) * curvature_weight
                } else {
                    0.0
                };
                let cost = edge_len * (1.0 + curv_penalty);
                if cost < best_cost {
                    best_cost = cost;
                    best_a = a;
                    best_b = b;
                }
            }
        }
        if best_cost.is_infinite() {
            break;
        }
        pts[best_a] = [
            (pts[best_a][0] + pts[best_b][0]) / 2.0,
            (pts[best_a][1] + pts[best_b][1]) / 2.0,
            (pts[best_a][2] + pts[best_b][2]) / 2.0,
        ];
        remap[best_b] = best_a;
        for ti in 0..tris.len() {
            if removed[ti] {
                continue;
            }
            let v = [
                res(tris[ti][0], &remap),
                res(tris[ti][1], &remap),
                res(tris[ti][2], &remap),
            ];
            if v[0] == v[1] || v[1] == v[2] || v[2] == v[0] {
                removed[ti] = true;
            }
        }
    }
    let remaining: Vec<[usize; 3]> = tris
        .iter()
        .enumerate()
        .filter(|(i, _)| !removed[*i])
        .map(|(_, t)| [res(t[0], &remap), res(t[1], &remap), res(t[2], &remap)])
        .collect();
    let mut used = vec![false; npts];
    for t in &remaining {
        for &v in t {
            used[v] = true;
        }
    }
    let mut pm = vec![0usize; npts];
    let mut np = Points::<f64>::new();
    for i in 0..npts {
        if used[i] {
            pm[i] = np.len();
            np.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &remaining {
        polys.push_cell(&[pm[t[0]] as i64, pm[t[1]] as i64, pm[t[2]] as i64]);
    }
    let mut r = PolyData::new();
    r.points = np;
    r.polys = polys;
    r
}
fn res(mut v: usize, r: &[usize]) -> usize {
    while r[v] != v {
        v = r[v];
    }
    v
}
fn estimate_curv(pts: &[[f64; 3]], tris: &[[usize; 3]], n: usize) -> Vec<f64> {
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for t in tris {
        for i in 0..3 {
            let a = t[i];
            let b = t[(i + 1) % 3];
            if !nb[a].contains(&b) {
                nb[a].push(b);
            }
            if !nb[b].contains(&a) {
                nb[b].push(a);
            }
        }
    }
    (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let k = nb[i].len() as f64;
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                lap[0] += pts[j][0] - pts[i][0];
                lap[1] += pts[j][1] - pts[i][1];
                lap[2] += pts[j][2] - pts[i][2];
            }
            (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt() / k
        })
        .collect()
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
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let r = adaptive_edge_collapse(&m, 1, 1.0);
        assert!(r.polys.num_cells() <= 3);
    }
}
