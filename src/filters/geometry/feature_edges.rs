use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Type of edge detected by the feature edges filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Edge used by only one polygon (mesh boundary).
    Boundary,
    /// Edge where the dihedral angle exceeds the feature angle.
    Feature,
    /// Edge shared by exactly two polygons (smooth interior).
    Manifold,
    /// Edge shared by more than two polygons (non-manifold).
    NonManifold,
}

/// Parameters for feature edge extraction.
pub struct FeatureEdgesParams {
    /// Angle threshold in degrees. Edges with dihedral angle greater than
    /// this are classified as feature edges. Default: 30.0
    pub feature_angle: f64,
    /// Include boundary edges in output. Default: true
    pub boundary_edges: bool,
    /// Include feature edges in output. Default: true
    pub feature_edges: bool,
    /// Include manifold (interior smooth) edges. Default: false
    pub manifold_edges: bool,
    /// Include non-manifold edges. Default: true
    pub non_manifold_edges: bool,
}

impl Default for FeatureEdgesParams {
    fn default() -> Self {
        Self {
            feature_angle: 30.0,
            boundary_edges: true,
            feature_edges: true,
            manifold_edges: false,
            non_manifold_edges: true,
        }
    }
}

/// Extract feature, boundary, manifold, and non-manifold edges from a PolyData.
///
/// Returns a PolyData containing line cells for the selected edge types.
pub fn feature_edges(input: &PolyData, params: &FeatureEdgesParams) -> PolyData {
    // Build edge -> (face_count, face0, face1). Stores the first two face
    // indices; higher counts only affect non-manifold classification.
    let mut edge_data: HashMap<(i64, i64), (usize, usize, usize)> = HashMap::new();
    let nc = input.polys.num_cells();
    let mut face_normals: Vec<[f64; 3]> = Vec::with_capacity(nc);

    for ci in 0..nc {
        let cell = input.polys.cell(ci);
        face_normals.push(polygon_normal(input, cell));
        let n = cell.len();
        for i in 0..n {
            let (a, b) = (cell[i], cell[(i + 1) % n]);
            let key = if a < b { (a, b) } else { (b, a) };
            let entry = edge_data.entry(key).or_insert((0, 0, 0));
            if entry.0 == 0 {
                entry.1 = ci;
            } else if entry.0 == 1 {
                entry.2 = ci;
            }
            entry.0 += 1;
        }
    }

    let cos_threshold = (params.feature_angle.to_radians()).cos();

    // Pre-count edges to allocate output
    let mut pts_flat: Vec<f64> = Vec::new();
    let mut line_conn: Vec<i64> = Vec::new();
    let mut pt_map: Vec<i64> = vec![-1; input.points.len()];
    let mut edge_types = Vec::new();

    for (&(a, b), &(count, f0, f1)) in &edge_data {
        let edge_type = if count == 1 {
            EdgeType::Boundary
        } else if count == 2 {
            let n1 = face_normals[f0];
            let n2 = face_normals[f1];
            let dot = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
            if dot <= cos_threshold {
                EdgeType::Feature
            } else {
                EdgeType::Manifold
            }
        } else {
            EdgeType::NonManifold
        };

        let include = match edge_type {
            EdgeType::Boundary => params.boundary_edges,
            EdgeType::Feature => params.feature_edges,
            EdgeType::Manifold => params.manifold_edges,
            EdgeType::NonManifold => params.non_manifold_edges,
        };

        if include {
            // Map points (inline, no HashMap)
            for &id in &[a, b] {
                let ui = id as usize;
                if pt_map[ui] < 0 {
                    pt_map[ui] = (pts_flat.len() / 3) as i64;
                    let p = input.points.get(ui);
                    pts_flat.push(p[0]);
                    pts_flat.push(p[1]);
                    pts_flat.push(p[2]);
                }
            }
            line_conn.push(pt_map[a as usize]);
            line_conn.push(pt_map[b as usize]);
            edge_types.push(match edge_type {
                EdgeType::Boundary => 0.0,
                EdgeType::NonManifold => 0.222222,
                EdgeType::Feature => 0.444444,
                EdgeType::Manifold => 0.666667,
            });
        }
    }

    let n_lines = line_conn.len() / 2;
    let offsets: Vec<i64> = (0..=n_lines).map(|i| (i * 2) as i64).collect();

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(pts_flat);
    pd.lines = CellArray::from_raw(offsets, line_conn);
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Edge Types",
            edge_types,
            1,
        )));
    pd
}

fn polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    // Newell's method
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;
    let n = cell.len();
    for i in 0..n {
        let p = input.points.get(cell[i] as usize);
        let q = input.points.get(cell[(i + 1) % n] as usize);
        nx += (p[1] - q[1]) * (p[2] + q[2]);
        ny += (p[2] - q[2]) * (p[0] + q[0]);
        nz += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-10 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_edges_of_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = feature_edges(&pd, &FeatureEdgesParams::default());
        // Single triangle: all 3 edges are boundary
        assert_eq!(result.lines.num_cells(), 3);
    }

    #[test]
    fn shared_edge_not_boundary() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let params = FeatureEdgesParams {
            boundary_edges: true,
            feature_edges: false,
            manifold_edges: false,
            non_manifold_edges: false,
            ..Default::default()
        };
        let result = feature_edges(&pd, &params);
        // 2 triangles share edge (0,1), so 4 boundary edges remain
        assert_eq!(result.lines.num_cells(), 4);
    }

    #[test]
    fn feature_angle_detection() {
        // Two triangles meeting at 90 degrees
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, -1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let params = FeatureEdgesParams {
            feature_angle: 45.0, // 90 deg > 45 deg threshold
            boundary_edges: false,
            feature_edges: true,
            manifold_edges: false,
            non_manifold_edges: false,
        };
        let result = feature_edges(&pd, &params);
        // The shared edge should be a feature edge
        assert_eq!(result.lines.num_cells(), 1);
    }
}
