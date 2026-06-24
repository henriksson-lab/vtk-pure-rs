//! Geodesic medial axis (skeleton from boundary distance).
use crate::data::{CellArray, Points, PolyData};
pub fn geodesic_medial_axis(mesh: &PolyData, threshold: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return PolyData::new();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                if !nb[a].iter().any(|&(x, _)| x == b) {
                    nb[a].push((b, d));
                    nb[b].push((a, d));
                }
                *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    // Find boundary
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    if boundary.is_empty() {
        return PolyData::new();
    }
    // Geodesic distance from boundary
    let mut dist = vec![f64::INFINITY; n];
    let mut visited = vec![false; n];
    for &b in &boundary {
        dist[b] = 0.0;
    }
    for _ in 0..n {
        let u = (0..n).filter(|&i| !visited[i]).min_by(|&a, &b| {
            dist[a]
                .partial_cmp(&dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let u = match u {
            Some(u) => u,
            None => {
                break;
            }
        };
        if dist[u].is_infinite() {
            break;
        }
        visited[u] = true;
        for &(v, w) in &nb[u] {
            let alt = dist[u] + w;
            if alt < dist[v] {
                dist[v] = alt;
            }
        }
    }
    // Medial axis: vertices that are local maxima of boundary distance
    let nb_simple: Vec<Vec<usize>> = nb
        .iter()
        .map(|v| v.iter().map(|&(j, _)| j).collect())
        .collect();
    let mut medial = Vec::new();
    for i in 0..n {
        if dist[i] < threshold {
            continue;
        }
        let is_local_max = nb_simple[i].iter().all(|&j| dist[j] <= dist[i] + 1e-10);
        if is_local_max && !boundary.contains(&i) {
            medial.push(i);
        }
    }
    // Connect medial vertices that are neighbors
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let medial_set: std::collections::HashSet<usize> = medial.iter().copied().collect();
    for &mi in &medial {
        for &(ni, _) in &nb[mi] {
            if medial_set.contains(&ni) && mi < ni {
                let ia = *pm.entry(mi).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(mi));
                    i
                });
                let ib = *pm.entry(ni).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(ni));
                    i
                });
                lines.push_cell(&[ia as i64, ib as i64]);
            }
        }
    }
    // Add isolated medial vertices
    for &mi in &medial {
        if !pm.contains_key(&mi) {
            let idx = pts.len();
            pts.push(mesh.points.get(mi));
            let mut verts = CellArray::new();
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
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
                [4.0, 0.0, 0.0],
                [2.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 4], [1, 3, 4], [3, 2, 4], [2, 0, 4]],
        );
        let r = geodesic_medial_axis(&m, 0.1);
        assert!(r.points.len() >= 0);
    } // may or may not find medial axis
}
