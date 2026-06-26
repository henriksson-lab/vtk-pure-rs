//! Distance map computation on meshes.
//!
//! Compute geodesic distance fields, distance-to-boundary, and
//! signed distance from mesh surface.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute BFS hop distance from a set of seed vertices.
///
/// Distance is measured in number of edge hops, not Euclidean distance.
pub fn hop_distance_from_seeds(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut dist = vec![f64::MAX; n];

    let mut queue = std::collections::VecDeque::new();
    for &s in seeds {
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

    // Replace MAX with -1
    for d in &mut dist {
        if *d == f64::MAX {
            *d = -1.0;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HopDistance",
            dist,
            1,
        )));
    result
}

/// Compute weighted geodesic distance from seeds using Dijkstra.
pub fn geodesic_distance_from_seeds(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut dist = vec![f64::MAX; n];

    // Min-heap: (distance, vertex)
    let mut heap = std::collections::BinaryHeap::new();
    for &s in seeds {
        if s < n {
            dist[s] = 0.0;
            heap.push(std::cmp::Reverse((OrdF64(0.0), s)));
        }
    }

    while let Some(std::cmp::Reverse((OrdF64(d), v))) = heap.pop() {
        if d > dist[v] {
            continue;
        }
        for &nb in &adj[v] {
            let pv = mesh.points.get(v);
            let pn = mesh.points.get(nb);
            let edge_len =
                ((pv[0] - pn[0]).powi(2) + (pv[1] - pn[1]).powi(2) + (pv[2] - pn[2]).powi(2))
                    .sqrt();
            let new_d = d + edge_len;
            if new_d < dist[nb] {
                dist[nb] = new_d;
                heap.push(std::cmp::Reverse((OrdF64(new_d), nb)));
            }
        }
    }

    for d in &mut dist {
        if *d == f64::MAX {
            *d = -1.0;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            dist,
            1,
        )));
    result
}

/// Compute distance from each vertex to the nearest boundary edge.
pub fn distance_to_boundary(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let boundary_verts = find_boundary_vertices(mesh, n);
    let seeds: Vec<usize> = boundary_verts
        .iter()
        .enumerate()
        .filter(|(_, &b)| b)
        .map(|(i, _)| i)
        .collect();
    geodesic_distance_from_seeds(mesh, &seeds)
}

#[derive(Clone, Copy, PartialEq)]
struct OrdF64(f64);
impl Eq for OrdF64 {}
impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for OrdF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let (Ok(a), Ok(b)) = (
                usize::try_from(cell[i]),
                usize::try_from(cell[(i + 1) % nc]),
            ) else {
                continue;
            };
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn find_boundary_vertices(mesh: &PolyData, n: usize) -> Vec<bool> {
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let (Ok(a), Ok(b)) = (
                usize::try_from(cell[i]),
                usize::try_from(cell[(i + 1) % nc]),
            ) else {
                continue;
            };
            if a >= n || b >= n {
                continue;
            }
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut boundary = vec![false; n];
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary[a] = true;
            boundary[b] = true;
        }
    }
    boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> PolyData {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn hop_from_corner() {
        let mesh = make_grid();
        let result = hop_distance_from_seeds(&mesh, &[0]);
        let arr = result.point_data().get_array("HopDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn geodesic() {
        let mesh = make_grid();
        let result = geodesic_distance_from_seeds(&mesh, &[0]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn boundary_dist() {
        let mesh = make_grid();
        let result = distance_to_boundary(&mesh);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        // Corner vertex is on boundary → distance 0
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        // Center vertex (12 = row 2, col 2) should have positive distance
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] > 0.0);
    }
}
