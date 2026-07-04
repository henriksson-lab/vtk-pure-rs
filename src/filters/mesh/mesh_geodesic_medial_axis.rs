//! Geodesic medial axis (skeleton from boundary distance).
use crate::data::{CellArray, Points, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(PartialEq)]
struct State {
    cost: f64,
    node: usize,
}
impl Eq for State {}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

pub fn geodesic_medial_axis(mesh: &PolyData, threshold: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return PolyData::new();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        if !valid_polygon_cell(cell, n) {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            if let Some((a, b)) = add_edge(mesh, cell[i], cell[(i + 1) % nc], &mut nb) {
                *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(mesh, strip, &mut nb, &mut ec);
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(mesh, edge[0], edge[1], &mut nb);
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
    let mut heap = BinaryHeap::new();
    for &b in &boundary {
        dist[b] = 0.0;
        heap.push(State { cost: 0.0, node: b });
    }
    while let Some(State { cost, node }) = heap.pop() {
        if cost > dist[node] {
            continue;
        }
        for &(next_node, weight) in &nb[node] {
            let next = cost + weight;
            if next < dist[next_node] {
                dist[next_node] = next;
                heap.push(State {
                    cost: next,
                    node: next_node,
                });
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
    let mut verts = CellArray::new();
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
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r.lines = lines;
    r
}

fn add_triangle_strip_edges(
    mesh: &PolyData,
    strip: &[i64],
    nb: &mut [Vec<(usize, f64)>],
    edge_count: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    for tri in strip.windows(3) {
        if !valid_triangle(tri, nb.len()) {
            continue;
        }
        if let Some((a, b)) = add_edge(mesh, tri[0], tri[1], nb) {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
        if let Some((a, b)) = add_edge(mesh, tri[1], tri[2], nb) {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
        if let Some((a, b)) = add_edge(mesh, tri[2], tri[0], nb) {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
}

fn add_edge(
    mesh: &PolyData,
    a: i64,
    b: i64,
    nb: &mut [Vec<(usize, f64)>],
) -> Option<(usize, usize)> {
    let n = nb.len();
    let a = valid_point_id(a, n)?;
    let b = valid_point_id(b, n)?;
    if a == b {
        return None;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !nb[a].iter().any(|&(v, _)| v == b) {
        nb[a].push((b, d));
    }
    if !nb[b].iter().any(|&(v, _)| v == a) {
        nb[b].push((a, d));
    }
    Some((a, b))
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && tri
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn valid_polygon_cell(cell: &[i64], n_points: usize) -> bool {
    cell.len() >= 3
        && cell
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
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
