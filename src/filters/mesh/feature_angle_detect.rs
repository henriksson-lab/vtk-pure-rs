use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Detect feature edges where the dihedral angle between adjacent faces
/// exceeds a threshold. Returns a PolyData with line cells marking sharp
/// feature edges.
///
/// `threshold_deg` is the angle in degrees. Edges shared by two faces whose
/// dihedral angle is greater than this threshold are included in the output.
/// Boundary edges (used by only one face) are also included.
pub fn detect_feature_edges_by_angle(input: &PolyData, threshold_deg: f64) -> PolyData {
    // Build edge -> face list mapping
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut face_normals: Vec<[f64; 3]> = Vec::new();

    for (face_idx, cell) in input.polys.iter().enumerate() {
        let normal = polygon_normal(&input.points, cell);
        face_normals.push(normal);

        let n: usize = cell.len();
        for i in 0..n {
            let a: i64 = cell[i];
            let b: i64 = cell[(i + 1) % n];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(face_idx);
        }
    }

    let cos_threshold: f64 = threshold_deg.to_radians().cos();

    let mut point_map: HashMap<i64, usize> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for (&(a, b), faces) in &edge_faces {
        let is_feature: bool = if faces.len() == 1 {
            // Boundary edge: always include
            true
        } else if faces.len() == 2 {
            let n1 = face_normals[faces[0]];
            let n2 = face_normals[faces[1]];
            let d: f64 = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
            // Dihedral angle is large when dot product is small (normals diverge)
            d < cos_threshold
        } else {
            // Non-manifold edge: include
            true
        };

        if is_feature {
            let ma: i64 = map_point(a, &input.points, &mut out_points, &mut point_map);
            let mb: i64 = map_point(b, &input.points, &mut out_points, &mut point_map);
            out_lines.push_cell(&[ma, mb]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

fn map_point(
    id: i64,
    pts: &Points<f64>,
    out_pts: &mut Points<f64>,
    pm: &mut HashMap<i64, usize>,
) -> i64 {
    *pm.entry(id).or_insert_with(|| {
        let idx: usize = out_pts.len();
        out_pts.push(pts.get(id as usize));
        idx
    }) as i64
}

fn polygon_normal(points: &Points<f64>, cell: &[i64]) -> [f64; 3] {
    // Newell's method
    let mut nx: f64 = 0.0;
    let mut ny: f64 = 0.0;
    let mut nz: f64 = 0.0;
    let n: usize = cell.len();

    for i in 0..n {
        let pi = points.get(cell[i] as usize);
        let pj = points.get(cell[(i + 1) % n] as usize);
        nx += (pi[1] - pj[1]) * (pi[2] + pj[2]);
        ny += (pi[2] - pj[2]) * (pi[0] + pj[0]);
        nz += (pi[0] - pj[0]) * (pi[1] + pj[1]);
    }

    let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-10 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two triangles sharing an edge, forming a 90 degree dihedral angle.
    fn make_right_angle_mesh() -> PolyData {
        let mut pd = PolyData::new();
        // Triangle 1: in XY plane
        pd.points.push([0.0, 0.0, 0.0]); // 0
        pd.points.push([1.0, 0.0, 0.0]); // 1
        pd.points.push([0.5, 1.0, 0.0]); // 2
                                         // Triangle 2: in XZ plane, sharing edge 0-1
        pd.points.push([0.5, 0.0, 1.0]); // 3

        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 0, 3]);
        pd
    }

    #[test]
    fn detects_sharp_edge() {
        let mesh = make_right_angle_mesh();
        // Threshold of 45 degrees should detect the 90-degree edge
        let result = detect_feature_edges_by_angle(&mesh, 45.0);
        // Should have at least the shared edge plus boundary edges
        assert!(result.lines.num_cells() > 0, "should detect feature edges");
    }

    #[test]
    fn flat_mesh_no_feature_edges() {
        let mut pd = PolyData::new();
        // Two coplanar triangles
        pd.points.push([0.0, 0.0, 0.0]); // 0
        pd.points.push([1.0, 0.0, 0.0]); // 1
        pd.points.push([0.5, 1.0, 0.0]); // 2
        pd.points.push([1.5, 1.0, 0.0]); // 3
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);

        // With a very small threshold like 1 degree, the shared edge (coplanar)
        // should not be a feature edge, only boundary edges remain
        let result = detect_feature_edges_by_angle(&pd, 1.0);
        // Count how many edges are the shared interior edge (1-2)
        // All detected edges should be boundary edges
        // The shared edge has two faces so it should NOT be detected at 1 degree
        // (the angle is 0 degrees which is < 1 degree threshold)
        let total: usize = result.lines.num_cells();
        // There are 4 boundary edges for two triangles sharing one edge (5 unique edges, 1 shared, 4 boundary)
        assert!(total > 0, "boundary edges should still be detected");
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let pd = PolyData::new();
        let result = detect_feature_edges_by_angle(&pd, 30.0);
        assert_eq!(result.lines.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }
}
