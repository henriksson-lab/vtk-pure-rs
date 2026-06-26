//! Geodesic-based mesh partitioning: Voronoi on mesh surface.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Partition a mesh into regions by geodesic Voronoi from seed vertices.
///
/// Each vertex is assigned to the nearest seed via Dijkstra.
pub fn geodesic_voronoi_partition(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || seeds.is_empty() {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);

    let mut dist = vec![f64::MAX; n];
    let mut labels = vec![usize::MAX; n];
    let mut heap = std::collections::BinaryHeap::new();

    for (si, &seed) in seeds.iter().enumerate() {
        if seed < n {
            dist[seed] = 0.0;
            labels[seed] = si;
            heap.push(std::cmp::Reverse((OrdF64(0.0), seed, si)));
        }
    }

    while let Some(std::cmp::Reverse((OrdF64(d), v, label))) = heap.pop() {
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
                labels[nb] = label;
                heap.push(std::cmp::Reverse((OrdF64(new_d), nb, label)));
            }
        }
    }

    let label_data: Vec<f64> = labels
        .iter()
        .map(|&l| if l < usize::MAX { l as f64 } else { -1.0 })
        .collect();
    let dist_data: Vec<f64> = dist
        .iter()
        .map(|&d| if d < f64::MAX { d } else { -1.0 })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PartitionId",
            label_data,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            dist_data,
            1,
        )));
    result
}

/// Iteratively refine partition seeds to create centroidal Voronoi tessellation.
pub fn centroidal_voronoi_iterate(
    mesh: &PolyData,
    initial_seeds: &[usize],
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || initial_seeds.is_empty() {
        return mesh.clone();
    }
    let mut seeds = initial_seeds.to_vec();

    for _ in 0..iterations {
        let partitioned = geodesic_voronoi_partition(mesh, &seeds);
        let labels = partitioned.point_data().get_array("PartitionId").unwrap();
        let mut buf = [0.0f64];

        // Find centroid of each partition (closest vertex to geometric center)
        let n_parts = seeds.len();
        let mut centroids = vec![[0.0; 3]; n_parts];
        let mut counts = vec![0usize; n_parts];
        for i in 0..n {
            labels.tuple_as_f64(i, &mut buf);
            let label = buf[0] as usize;
            if label < n_parts {
                let p = mesh.points.get(i);
                for c in 0..3 {
                    centroids[label][c] += p[c];
                }
                counts[label] += 1;
            }
        }
        for pi in 0..n_parts {
            if counts[pi] > 0 {
                for c in 0..3 {
                    centroids[pi][c] /= counts[pi] as f64;
                }
            }
        }

        // Update seeds to closest vertex to centroid
        for pi in 0..n_parts {
            let mut best = seeds[pi];
            let mut best_d = f64::MAX;
            for i in 0..n {
                labels.tuple_as_f64(i, &mut buf);
                if buf[0] as usize != pi {
                    continue;
                }
                let p = mesh.points.get(i);
                let d = (p[0] - centroids[pi][0]).powi(2)
                    + (p[1] - centroids[pi][1]).powi(2)
                    + (p[2] - centroids[pi][2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            seeds[pi] = best;
        }
    }

    geodesic_voronoi_partition(mesh, &seeds)
}

#[derive(Clone, Copy, PartialEq)]
struct OrdF64(f64);
impl Eq for OrdF64 {}
impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&o.0)
    }
}
impl Ord for OrdF64 {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.partial_cmp(o).unwrap_or(std::cmp::Ordering::Equal)
    }
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
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            let a = edge[0] as usize;
            let b = edge[1] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn voronoi_partition() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = geodesic_voronoi_partition(&mesh, &[0, 99]);
        assert!(result.point_data().get_array("PartitionId").is_some());
        assert!(result.point_data().get_array("GeodesicDistance").is_some());

        let arr = result.point_data().get_array("PartitionId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(99, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn cvt() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..7 {
            for x in 0..7 {
                let bl = y * 8 + x;
                tris.push([bl, bl + 1, bl + 9]);
                tris.push([bl, bl + 9, bl + 8]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = centroidal_voronoi_iterate(&mesh, &[0, 63], 3);
        assert!(result.point_data().get_array("PartitionId").is_some());
    }
}
