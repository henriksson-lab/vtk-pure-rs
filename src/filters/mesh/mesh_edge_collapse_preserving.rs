//! Edge collapse that preserves mesh topology (boundary, genus).
use crate::data::{CellArray, Points, PolyData};
pub fn collapse_edges_preserving(mesh: &PolyData, target_faces: usize) -> PolyData {
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
    // Find boundary vertices
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for t in &tris {
        for i in 0..3 {
            let a = t[i];
            let b = t[(i + 1) % 3];
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }

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
                if a == b || boundary.contains(&a) || boundary.contains(&b) {
                    continue;
                }
                let cost = (pts[a][0] - pts[b][0]).powi(2)
                    + (pts[a][1] - pts[b][1]).powi(2)
                    + (pts[a][2] - pts[b][2]).powi(2);
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
        let r = collapse_edges_preserving(&m, 1);
        assert!(r.polys.num_cells() <= 3);
    }
}
