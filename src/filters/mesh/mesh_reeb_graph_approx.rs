//! Approximate Reeb graph from scalar function on mesh.
use crate::data::{CellArray, Points, PolyData};
pub fn reeb_graph(mesh: &PolyData, array_name: &str, num_levels: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == mesh.points.len() => a,
        _ => return PolyData::new(),
    };
    let n = mesh.points.len();
    if n == 0 {
        return PolyData::new();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_neighbor_pair(&mut nb, n, cell[i], cell[(i + 1) % nc]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_pair(&mut nb, n, edge[0], edge[1]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_neighbor_pair(&mut nb, n, tri[0], tri[1]);
            add_neighbor_pair(&mut nb, n, tri[1], tri[2]);
            add_neighbor_pair(&mut nb, n, tri[2], tri[0]);
        }
    }
    let nl = num_levels.max(3);
    // At each level, find connected components; Reeb graph nodes = components, edges = merges/splits
    let mut level_components: Vec<Vec<(usize, f64, f64, f64)>> = Vec::new(); // (root, cx, cy, cz)
    for li in 0..=nl {
        let lo = mn + range * li as f64 / nl as f64;
        let hi = mn + range * (li + 1).min(nl) as f64 / nl as f64;
        let band: Vec<bool> = vals.iter().map(|&v| v >= lo && v <= hi).collect();
        let mut parent: Vec<usize> = (0..n).collect();
        for i in 0..n {
            if !band[i] {
                continue;
            }
            for &j in &nb[i] {
                if band[j] {
                    union(&mut parent, i, j);
                }
            }
        }
        let mut comp: std::collections::HashMap<usize, (f64, f64, f64, usize)> =
            std::collections::HashMap::new();
        for i in 0..n {
            if !band[i] {
                continue;
            }
            let root = find(&mut parent, i);
            let p = mesh.points.get(i);
            let e = comp.entry(root).or_insert((0.0, 0.0, 0.0, 0));
            e.0 += p[0];
            e.1 += p[1];
            e.2 += p[2];
            e.3 += 1;
        }
        let nodes: Vec<(usize, f64, f64, f64)> = comp
            .iter()
            .map(|(&r, &(cx, cy, cz, c))| {
                let cf = c as f64;
                (r, cx / cf, cy / cf, cz / cf)
            })
            .collect();
        level_components.push(nodes);
    }
    // Build Reeb graph: nodes are component centroids, edges connect adjacent levels
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut level_indices: Vec<Vec<usize>> = Vec::new();
    for nodes in &level_components {
        let mut idxs = Vec::new();
        for &(_, cx, cy, cz) in nodes {
            idxs.push(pts.len());
            pts.push([cx, cy, cz]);
        }
        level_indices.push(idxs);
    }
    // Connect nodes between adjacent levels by proximity
    for li in 0..level_indices.len() - 1 {
        for &i in &level_indices[li] {
            for &j in &level_indices[li + 1] {
                let pi = pts.get(i);
                let pj = pts.get(j);
                let d =
                    ((pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2))
                        .sqrt();
                if d < range * 2.0 {
                    lines.push_cell(&[i as i64, j as i64]);
                }
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra != rb {
        p[rb] = ra;
    }
}

fn add_neighbor_pair(nb: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

fn valid_point_id(point_id: i64, npoints: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&id| id < npoints)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 2.0, 1.5],
                1,
            )));
        let r = reeb_graph(&m, "h", 5);
        assert!(r.points.len() >= 2);
    }
}
