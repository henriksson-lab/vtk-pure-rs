//! Boundary analysis: extract boundary loops, measure perimeters.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract all boundary loops as separate polylines.
pub fn extract_boundary_loops(mesh: &PolyData) -> Vec<PolyData> {
    let loops = find_boundary_loops(mesh);
    loops
        .iter()
        .map(|loop_v| {
            let mut pts = Points::<f64>::new();
            let mut lines = CellArray::new();
            let ids: Vec<i64> = loop_v
                .iter()
                .enumerate()
                .map(|(i, &vi)| {
                    pts.push(mesh.points.get(vi));
                    i as i64
                })
                .collect();
            if ids.len() >= 2 {
                let mut closed = ids.clone();
                closed.push(ids[0]);
                lines.push_cell(&closed);
            }
            let mut m = PolyData::new();
            m.points = pts;
            m.lines = lines;
            m
        })
        .collect()
}

/// Compute the perimeter of each boundary loop.
pub fn boundary_perimeters(mesh: &PolyData) -> Vec<f64> {
    let loops = find_boundary_loops(mesh);
    loops
        .iter()
        .map(|loop_v| {
            let mut perim = 0.0;
            for i in 0..loop_v.len() {
                let a = mesh.points.get(loop_v[i]);
                let b = mesh.points.get(loop_v[(i + 1) % loop_v.len()]);
                perim +=
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            }
            perim
        })
        .collect()
}

/// Classify boundary type: "open" (has boundary) or "closed" (no boundary).
pub fn boundary_classification(mesh: &PolyData) -> &'static str {
    if find_boundary_loops(mesh).is_empty() {
        "closed"
    } else {
        "open"
    }
}

/// Add a "BoundaryDistance" point data: hop distance from nearest boundary vertex.
pub fn boundary_distance_field(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let loops = find_boundary_loops(mesh);
    let seeds: Vec<usize> = loops.into_iter().flatten().collect();
    if seeds.is_empty() {
        let mut result = mesh.clone();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "BoundaryDistance",
                vec![0.0; n],
                1,
            )));
        return result;
    }

    let adj = build_adj(mesh, n);
    let mut dist = vec![f64::MAX; n];
    let mut queue = std::collections::VecDeque::new();
    for &s in &seeds {
        if s < n {
            dist[s] = 0.0;
            queue.push_back(s);
        }
    }
    while let Some(v) = queue.pop_front() {
        let d = dist[v] + 1.0;
        for &nb in &adj[v] {
            if d < dist[nb] {
                dist[nb] = d;
                queue.push_back(nb);
            }
        }
    }
    for d in &mut dist {
        if *d == f64::MAX {
            *d = 0.0;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryDistance",
            dist,
            1,
        )));
    result
}

fn find_boundary_loops(mesh: &PolyData) -> Vec<Vec<usize>> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= mesh.points.len() || b >= mesh.points.len() {
                continue;
            }
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let bnd: Vec<(usize, usize)> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    if bnd.is_empty() {
        return Vec::new();
    }
    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &bnd {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    for nbs in adj.values_mut() {
        nbs.sort_unstable();
        nbs.dedup();
    }

    let mut visited_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    let mut loops = Vec::new();
    for &(a, b) in &bnd {
        let key = edge_key(a, b);
        if visited_edges.contains(&key) {
            continue;
        }
        let mut path = vec![a, b];
        visited_edges.insert(key);
        extend_boundary_path(&adj, &mut visited_edges, &mut path, false);
        extend_boundary_path(&adj, &mut visited_edges, &mut path, true);
        if path.len() > 1 && path[0] == *path.last().unwrap() {
            path.pop();
        }
        if path.len() >= 3 {
            loops.push(path);
        }
    }
    loops
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn extend_boundary_path(
    adj: &std::collections::HashMap<usize, Vec<usize>>,
    visited_edges: &mut std::collections::HashSet<(usize, usize)>,
    path: &mut Vec<usize>,
    prepend: bool,
) {
    loop {
        let (prev, cur) = if prepend {
            (path[1], path[0])
        } else {
            (path[path.len() - 2], path[path.len() - 1])
        };
        let Some(nbs) = adj.get(&cur) else {
            break;
        };
        let next = nbs
            .iter()
            .copied()
            .find(|&nb| nb != prev && !visited_edges.contains(&edge_key(cur, nb)));
        let Some(nb) = next else {
            break;
        };
        visited_edges.insert(edge_key(cur, nb));
        if prepend {
            path.insert(0, nb);
        } else {
            path.push(nb);
        }
    }
}

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_tri_boundary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let loops = extract_boundary_loops(&mesh);
        assert_eq!(loops.len(), 1);
        let perimeters = boundary_perimeters(&mesh);
        assert!(perimeters[0] > 2.0);
        assert_eq!(boundary_classification(&mesh), "open");
    }
    #[test]
    fn closed_tet() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        assert_eq!(boundary_classification(&mesh), "closed");
        assert!(extract_boundary_loops(&mesh).is_empty());
    }
    #[test]
    fn distance_field() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = boundary_distance_field(&mesh);
        let arr = result.point_data().get_array("BoundaryDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // corner is boundary
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] > 0.0); // center
    }
}
