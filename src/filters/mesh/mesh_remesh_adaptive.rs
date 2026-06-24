//! Adaptive remeshing based on curvature.
use crate::data::{CellArray, Points, PolyData};
pub fn remesh_by_curvature(
    mesh: &PolyData,
    min_edge: f64,
    max_edge: f64,
    iterations: usize,
) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let n = pts.len();
    // Estimate curvature per vertex
    let curv = estimate_curvature(&pts, &tris, n);
    for _ in 0..iterations {
        // Split long edges in high-curvature regions
        let mut new_tris = Vec::new();
        let mut em: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        for t in &tris {
            let v = [t[0], t[1], t[2]];
            let max_c = v
                .iter()
                .map(|&vi| if vi < curv.len() { curv[vi] } else { 0.0 })
                .fold(0.0f64, f64::max);
            let target = max_edge - (max_edge - min_edge) * max_c.min(1.0);
            let mut longest_idx = 0;
            let mut longest_d = 0.0f64;
            for i in 0..3 {
                let d = edge_len(&pts[v[i]], &pts[v[(i + 1) % 3]]);
                if d > longest_d {
                    longest_d = d;
                    longest_idx = i;
                }
            }
            if longest_d > target {
                let a = v[longest_idx];
                let b = v[(longest_idx + 1) % 3];
                let c = v[(longest_idx + 2) % 3];
                let key = (a.min(b), a.max(b));
                let mid = *em.entry(key).or_insert_with(|| {
                    let i = pts.len();
                    pts.push([
                        (pts[a][0] + pts[b][0]) / 2.0,
                        (pts[a][1] + pts[b][1]) / 2.0,
                        (pts[a][2] + pts[b][2]) / 2.0,
                    ]);
                    i
                });
                new_tris.push([a, mid, c]);
                new_tris.push([mid, b, c]);
            } else {
                new_tris.push(*t);
            }
        }
        // Collapse short edges
        let npts = pts.len();
        let mut remap: Vec<usize> = (0..npts).collect();
        for t in &new_tris {
            for i in 0..3 {
                let a = t[i];
                let b = t[(i + 1) % 3];
                if a == b {
                    continue;
                }
                let d = edge_len(&pts[a], &pts[b]);
                if d < min_edge * 0.8 {
                    pts[a] = [
                        (pts[a][0] + pts[b][0]) / 2.0,
                        (pts[a][1] + pts[b][1]) / 2.0,
                        (pts[a][2] + pts[b][2]) / 2.0,
                    ];
                    remap[b] = a;
                }
            }
        }
        tris = new_tris
            .iter()
            .map(|t| [res(t[0], &remap), res(t[1], &remap), res(t[2], &remap)])
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[2] != t[0])
            .collect();
    }
    let mut used = vec![false; pts.len()];
    for t in &tris {
        for &v in t {
            used[v] = true;
        }
    }
    let mut pm = vec![0usize; pts.len()];
    let mut np = Points::<f64>::new();
    for i in 0..pts.len() {
        if used[i] {
            pm[i] = np.len();
            np.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &tris {
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
fn edge_len(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn estimate_curvature(pts: &[[f64; 3]], tris: &[[usize; 3]], n: usize) -> Vec<f64> {
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
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = remesh_by_curvature(&m, 1.0, 5.0, 2);
        assert!(r.polys.num_cells() >= 1);
    }
}
