use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Simplify a triangle mesh using quadric error metric edge collapse.
///
/// `target_reduction` is the fraction of triangles to remove (0.0 = none, 1.0 = all).
/// Uses adjacency structures for O(1) neighbor lookups per collapse.
pub fn decimate(input: &PolyData, target_reduction: f64) -> PolyData {
    let target_reduction = target_reduction.clamp(0.0, 0.99);
    let n_points = input.points.len();
    let n_cells = input.polys.num_cells();

    if target_reduction == 0.0 || n_cells == 0 || n_points < 4 {
        return input.clone();
    }

    let target_cells = ((1.0 - target_reduction) * n_cells as f64).ceil() as usize;
    let target_cells = target_cells.max(1);

    // Copy points
    let mut points: Vec<[f64; 3]> = (0..n_points).map(|i| input.points.get(i)).collect();

    if input.polys.iter().any(|cell| cell.len() != 3) {
        return input.clone();
    }

    // Build triangle list
    let mut triangles: Vec<[usize; 3]> = Vec::with_capacity(n_cells);
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            triangles.push([cell[0] as usize, cell[1] as usize, cell[2] as usize]);
        }
    }
    let mut tri_alive = vec![true; triangles.len()];

    // Adjacency: vertex → set of triangle indices
    let mut vert_tris: Vec<HashSet<usize>> = vec![HashSet::new(); n_points];
    for (ti, tri) in triangles.iter().enumerate() {
        for &v in tri {
            vert_tris[v].insert(ti);
        }
    }

    // Compute quadric matrices per vertex (symmetric 4x4 as 10 values)
    let mut quadrics = vec![[0.0f64; 10]; n_points];
    for tri in &triangles {
        let q = triangle_quadric(points[tri[0]], points[tri[1]], points[tri[2]]);
        for &v in tri {
            add_quadric(&mut quadrics[v], &q);
        }
    }

    // Edge priority queue
    let mut heap: BinaryHeap<EdgeCollapse> = BinaryHeap::new();
    let mut edges_seen = HashSet::new();
    let mut version = vec![0u32; n_points]; // invalidation counter per vertex

    for tri in &triangles {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let edge = if a < b { (a, b) } else { (b, a) };
            if edges_seen.insert(edge) {
                let cost = edge_cost(&quadrics[a], &quadrics[b], points[a], points[b]);
                heap.push(EdgeCollapse {
                    cost: -cost,
                    v0: edge.0,
                    v1: edge.1,
                    ver0: version[edge.0],
                    ver1: version[edge.1],
                });
            }
        }
    }

    let mut live_count = triangles.len();

    // Greedy collapse loop
    while live_count > target_cells {
        let Some(collapse) = heap.pop() else { break };

        let (a, b) = (collapse.v0, collapse.v1);

        // Skip stale entries (vertex was already modified)
        if collapse.ver0 != version[a] || collapse.ver1 != version[b] {
            continue;
        }

        // Merge b into a: update position and quadric
        points[a] = midpoint(points[a], points[b]);
        let qb = quadrics[b];
        add_quadric(&mut quadrics[a], &qb);
        version[a] += 1;
        version[b] += 1;

        // Collect triangles touching b
        let b_tris: Vec<usize> = vert_tris[b].iter().copied().collect();

        for &ti in &b_tris {
            if !tri_alive[ti] {
                continue;
            }
            let tri = &mut triangles[ti];

            // Replace b with a in this triangle
            for v in tri.iter_mut() {
                if *v == b {
                    *v = a;
                }
            }

            // Check degenerate (two vertices the same)
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                tri_alive[ti] = false;
                live_count -= 1;
                // Remove from adjacency
                for &v in triangles[ti].iter() {
                    vert_tris[v].remove(&ti);
                }
            } else {
                // Move triangle from b's adjacency to a's
                vert_tris[b].remove(&ti);
                vert_tris[a].insert(ti);
            }
        }

        // Re-insert edges touching a with fresh costs
        let mut new_edges = HashSet::new();
        for &ti in &vert_tris[a] {
            if !tri_alive[ti] {
                continue;
            }
            let tri = triangles[ti];
            for &v in &tri {
                if v != a {
                    let edge = if a < v { (a, v) } else { (v, a) };
                    if new_edges.insert(edge) {
                        let cost = edge_cost(&quadrics[a], &quadrics[v], points[a], points[v]);
                        heap.push(EdgeCollapse {
                            cost: -cost,
                            v0: edge.0,
                            v1: edge.1,
                            ver0: version[edge.0],
                            ver1: version[edge.1],
                        });
                    }
                }
            }
        }

        if live_count <= target_cells {
            break;
        }
    }

    // Build output: compact points and triangles
    let mut point_map = vec![usize::MAX; points.len()];
    let mut old_point_ids = Vec::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for (ti, tri) in triangles.iter().enumerate() {
        if !tri_alive[ti] {
            continue;
        }
        for &v in tri {
            if point_map[v] == usize::MAX {
                point_map[v] = out_points.len();
                old_point_ids.push(v);
                out_points.push(points[v]);
            }
        }
        out_polys.push_cell(&[
            point_map[tri[0]] as i64,
            point_map[tri[1]] as i64,
            point_map[tri[2]] as i64,
        ]);
    }

    let mut output = PolyData::new();
    output.points = out_points;
    output.polys = out_polys;
    for array in input.point_data().iter() {
        if let Some(subset) = subset_array(array, &old_point_ids) {
            output.point_data_mut().add_array(subset);
        }
    }
    output
}

#[derive(Debug)]
struct EdgeCollapse {
    cost: f64,
    v0: usize,
    v1: usize,
    ver0: u32,
    ver1: u32,
}

impl PartialEq for EdgeCollapse {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}
impl Eq for EdgeCollapse {}
impl PartialOrd for EdgeCollapse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for EdgeCollapse {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost
            .partial_cmp(&other.cost)
            .unwrap_or(Ordering::Equal)
    }
}

fn triangle_quadric(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> [f64; 10] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-10 {
        return [0.0; 10];
    }
    let (a, b, c) = (n[0] / len, n[1] / len, n[2] / len);
    let d = -(a * p0[0] + b * p0[1] + c * p0[2]);
    [
        a * a,
        a * b,
        a * c,
        a * d,
        b * b,
        b * c,
        b * d,
        c * c,
        c * d,
        d * d,
    ]
}

fn add_quadric(a: &mut [f64; 10], b: &[f64; 10]) {
    for i in 0..10 {
        a[i] += b[i];
    }
}

fn edge_cost(q0: &[f64; 10], q1: &[f64; 10], p0: [f64; 3], p1: [f64; 3]) -> f64 {
    let mut q = [0.0; 10];
    for i in 0..10 {
        q[i] = q0[i] + q1[i];
    }
    let mp = midpoint(p0, p1);
    let (x, y, z) = (mp[0], mp[1], mp[2]);
    q[0] * x * x
        + 2.0 * q[1] * x * y
        + 2.0 * q[2] * x * z
        + 2.0 * q[3] * x
        + q[4] * y * y
        + 2.0 * q[5] * y * z
        + 2.0 * q[6] * y
        + q[7] * z * z
        + 2.0 * q[8] * z
        + q[9]
}

fn midpoint(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            let nc = a.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                if tuple_id >= a.num_tuples() {
                    return None;
                }
                data.extend_from_slice(a.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                a.name(),
                data,
                nc,
            )))
        }};
    }

    match array {
        AnyDataArray::F32(_) => subset_variant!(F32),
        AnyDataArray::F64(_) => subset_variant!(F64),
        AnyDataArray::I8(_) => subset_variant!(I8),
        AnyDataArray::I16(_) => subset_variant!(I16),
        AnyDataArray::I32(_) => subset_variant!(I32),
        AnyDataArray::I64(_) => subset_variant!(I64),
        AnyDataArray::U8(_) => subset_variant!(U8),
        AnyDataArray::U16(_) => subset_variant!(U16),
        AnyDataArray::U32(_) => subset_variant!(U32),
        AnyDataArray::U64(_) => subset_variant!(U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::core::sources;

    #[test]
    fn decimate_sphere() {
        let sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            theta_resolution: 16,
            phi_resolution: 16,
            ..Default::default()
        });
        let tri = crate::filters::geometry::triangulate::triangulate(&sphere);
        let original = tri.polys.num_cells();
        let decimated = decimate(&tri, 0.5);
        assert!(decimated.polys.num_cells() < original);
        assert!(decimated.polys.num_cells() > 0);
    }

    #[test]
    fn decimate_zero() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.add_scalars("s", vec![1.0, 2.0, 3.0]);
        let result = decimate(&pd, 0.0);
        assert_eq!(result.polys.num_cells(), 1);
        assert!(result.point_data().get_array("s").is_some());
    }

    #[test]
    fn decimate_non_triangles_passes_input_through() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.add_scalars("s", vec![1.0, 2.0, 3.0, 4.0]);

        let result = decimate(&pd, 0.5);

        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2, 3]);
        assert!(result.point_data().get_array("s").is_some());
    }

    #[test]
    fn decimate_maps_surviving_point_data() {
        let sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            theta_resolution: 8,
            phi_resolution: 8,
            ..Default::default()
        });
        let mut tri = crate::filters::geometry::triangulate::triangulate(&sphere);
        let values: Vec<f64> = (0..tri.points.len()).map(|i| i as f64).collect();
        tri.add_scalars("ids", values);

        let decimated = decimate(&tri, 0.5);
        let ids = decimated.point_data().get_array("ids").unwrap();

        assert_eq!(ids.num_tuples(), decimated.points.len());
    }
}
