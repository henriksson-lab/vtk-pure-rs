use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

/// Compute per-vertex edge valence (number of edges meeting at each vertex).
///
/// Adds an "EdgeValence" point data array (1-component, i32) to the output.
pub fn compute_edge_valence(input: &PolyData) -> PolyData {
    let num_pts: usize = input.points.len();
    let mut valence = vec![0i32; num_pts];

    for (a, b) in unique_edges(input) {
        valence[a] += 1;
        valence[b] += 1;
    }

    let valence_f64: Vec<f64> = valence.iter().map(|&v| v as f64).collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "EdgeValence",
            valence_f64,
            1,
        )));
    pd
}

fn unique_edges(input: &PolyData) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    let num_pts = input.points.len();

    for cell in input.lines.iter() {
        for pair in cell.windows(2) {
            insert_edge(&mut edges, pair[0], pair[1], num_pts);
        }
    }

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(&mut edges, cell[i], cell[(i + 1) % cell.len()], num_pts);
        }
    }

    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for tri in strip.windows(3) {
            insert_edge(&mut edges, tri[0], tri[1], num_pts);
            insert_edge(&mut edges, tri[1], tri[2], num_pts);
            insert_edge(&mut edges, tri[2], tri[0], num_pts);
        }
    }

    edges
}

fn insert_edge(edges: &mut HashSet<(usize, usize)>, a: i64, b: i64, num_pts: usize) {
    if a < 0 || b < 0 || a == b {
        return;
    }
    let a = a as usize;
    let b = b as usize;
    if a >= num_pts || b >= num_pts {
        return;
    }
    edges.insert(if a < b { (a, b) } else { (b, a) });
}

/// Statistics about edge valence across the whole mesh.
pub struct ValenceStats {
    pub min_valence: u32,
    pub max_valence: u32,
    pub mean_valence: f64,
}

/// Compute min, max, and mean edge valence over all vertices.
///
/// Returns `None` if the mesh has no points.
pub fn edge_valence_stats(input: &PolyData) -> Option<ValenceStats> {
    let num_pts: usize = input.points.len();
    if num_pts == 0 {
        return None;
    }

    let result = compute_edge_valence(input);
    let arr = result.point_data().get_array("EdgeValence").unwrap();

    let mut min_v: u32 = u32::MAX;
    let mut max_v: u32 = 0;
    let mut sum: f64 = 0.0;
    let mut buf = [0.0f64];

    for i in 0..num_pts {
        arr.tuple_as_f64(i, &mut buf);
        let v = buf[0] as u32;
        if v < min_v {
            min_v = v;
        }
        if v > max_v {
            max_v = v;
        }
        sum += buf[0];
    }

    Some(ValenceStats {
        min_valence: min_v,
        max_valence: max_v,
        mean_valence: sum / num_pts as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_valence() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_edge_valence(&pd);
        let arr = result.point_data().get_array("EdgeValence").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0] as u32, 2); // each vertex has 2 edges
        }
    }

    #[test]
    fn two_shared_edge_triangles() {
        // Two triangles sharing edge 0-1
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let stats = edge_valence_stats(&pd).unwrap();
        // Vertices 0 and 1 have 3 edges each, vertices 2 and 3 have 2 edges each
        assert_eq!(stats.min_valence, 2);
        assert_eq!(stats.max_valence, 3);
        // mean = (3+3+2+2)/4 = 2.5
        assert!((stats.mean_valence - 2.5).abs() < 1e-10);
    }

    #[test]
    fn polyline_edges_contribute_to_valence() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let stats = edge_valence_stats(&pd).unwrap();
        assert_eq!(stats.min_valence, 1);
        assert_eq!(stats.max_valence, 2);
        assert!((stats.mean_valence - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert!(edge_valence_stats(&pd).is_none());
    }
}
