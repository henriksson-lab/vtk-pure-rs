//! BoundaryMeshQuality — compute quality metrics for boundary edges.

use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Boundary edge quality statistics.
#[derive(Debug, Clone)]
pub struct BoundaryEdgeStats {
    pub min_length: f64,
    pub max_length: f64,
    pub mean_length: f64,
    pub num_boundary_edges: usize,
}

/// Compute quality metrics for boundary edges of a triangle mesh.
///
/// A boundary edge is one that appears in exactly one triangle.
/// Adds "BoundaryEdgeLength" and "BoundaryAngle" arrays to point data
/// for boundary vertices.
pub fn boundary_mesh_quality(input: &PolyData) -> PolyData {
    let n = input.points.len();

    // Count edge occurrences to find boundary edges
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for cell in input.polys.iter() {
        let nv = cell.len();
        for i in 0..nv {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nv] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Collect boundary edges
    let boundary_edges: Vec<(usize, usize)> = edge_count
        .iter()
        .filter(|&(_, &count)| count == 1)
        .map(|(&edge, _)| edge)
        .collect();

    // Build adjacency on boundary: for each boundary vertex, its boundary neighbors
    let mut boundary_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(a, b) in &boundary_edges {
        boundary_adj.entry(a).or_default().push(b);
        boundary_adj.entry(b).or_default().push(a);
    }

    // Compute edge lengths for boundary edges
    let mut edge_lengths = Vec::new();
    let mut point_edge_length = vec![0.0f64; n];
    let mut point_edge_count = vec![0usize; n];

    for &(a, b) in &boundary_edges {
        let pa = input.points.get(a);
        let pb = input.points.get(b);
        let len =
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        edge_lengths.push(len);
        point_edge_length[a] += len;
        point_edge_count[a] += 1;
        point_edge_length[b] += len;
        point_edge_count[b] += 1;
    }

    // Average edge length per boundary vertex
    let boundary_edge_length: Vec<f64> = (0..n)
        .map(|i| {
            if point_edge_count[i] > 0 {
                point_edge_length[i] / point_edge_count[i] as f64
            } else {
                0.0
            }
        })
        .collect();

    // Compute boundary angle at each boundary vertex
    let mut boundary_angle = vec![0.0f64; n];
    for (&v, neighbors) in &boundary_adj {
        if neighbors.len() == 2 {
            let pv = input.points.get(v);
            let p1 = input.points.get(neighbors[0]);
            let p2 = input.points.get(neighbors[1]);

            let d1 = [p1[0] - pv[0], p1[1] - pv[1], p1[2] - pv[2]];
            let d2 = [p2[0] - pv[0], p2[1] - pv[1], p2[2] - pv[2]];

            let dot = d1[0] * d2[0] + d1[1] * d2[1] + d1[2] * d2[2];
            let len1 = (d1[0] * d1[0] + d1[1] * d1[1] + d1[2] * d1[2]).sqrt();
            let len2 = (d2[0] * d2[0] + d2[1] * d2[1] + d2[2] * d2[2]).sqrt();

            if len1 > 1e-15 && len2 > 1e-15 {
                let cos_angle = (dot / (len1 * len2)).clamp(-1.0, 1.0);
                boundary_angle[v] = cos_angle.acos().to_degrees();
            }
        }
    }

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryEdgeLength",
            boundary_edge_length,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryAngle",
            boundary_angle,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_all_boundary() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = boundary_mesh_quality(&pd);
        let edge_arr = result.point_data().get_array("BoundaryEdgeLength").unwrap();
        let angle_arr = result.point_data().get_array("BoundaryAngle").unwrap();
        assert_eq!(edge_arr.num_tuples(), 3);
        assert_eq!(angle_arr.num_tuples(), 3);

        // Each vertex has 2 boundary neighbors, so boundary angle should be nonzero
        let mut buf = [0.0f64];
        angle_arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 0.0,
            "boundary angle should be positive at vertex 0"
        );

        // Vertex 0 is at origin, angle between edges to (1,0,0) and (0,1,0) = 90 degrees
        assert!(
            (buf[0] - 90.0).abs() < 1e-6,
            "expected 90 deg, got {}",
            buf[0]
        );
    }

    #[test]
    fn shared_edge_not_boundary() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let result = boundary_mesh_quality(&pd);
        let edge_arr = result.point_data().get_array("BoundaryEdgeLength").unwrap();
        // The shared edge 1-2 should not be boundary
        // All 4 vertices are boundary in this case since it's an open surface
        let mut buf = [0.0f64];
        edge_arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0);
    }
}
