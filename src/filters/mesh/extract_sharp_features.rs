use crate::data::{CellArray, PolyData};

/// Extract edges that lie on sharp features based on dihedral angle.
///
/// Re-exported from [`crate::filters::mesh::sharp_edges`], which holds the single
/// implementation. Note that it emits *only* edges shared by exactly two polygons
/// (`vtkFeatureEdges` with feature edges on and boundary/non-manifold edges off);
/// use [`crate::filters::geometry::feature_edges`] for the full VTK filter with
/// boundary and non-manifold edge categories.
pub use crate::filters::mesh::sharp_edges::extract_sharp_edges;

/// Extract vertices that lie on sharp features based on a curvature-like metric.
///
/// A vertex is considered "sharp" if the maximum angle between normals of its
/// adjacent faces exceeds `curvature_threshold` (in degrees). These vertices
/// are returned as a PolyData containing vertex cells.
pub fn extract_sharp_vertices(input: &PolyData, curvature_threshold: f64) -> PolyData {
    let n = input.points.len();
    let cos_thresh: f64 = curvature_threshold.to_radians().cos();

    // Build vertex -> face list mapping
    let mut vertex_faces: Vec<Vec<usize>> = vec![Vec::new(); n];
    let faces: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let mut face_normals: Vec<[f64; 3]> = Vec::with_capacity(faces.len());

    for (fi, cell) in faces.iter().enumerate() {
        let normal = polygon_normal(input, cell);
        face_normals.push(normal);
        for &vid in cell {
            vertex_faces[vid as usize].push(fi);
        }
    }

    let mut out = PolyData::new();
    let mut verts = CellArray::new();

    for i in 0..n {
        let adj = &vertex_faces[i];
        if adj.len() < 2 {
            continue;
        }
        // Check if any pair of adjacent face normals has a large angle
        let mut is_sharp: bool = false;
        'outer: for a in 0..adj.len() {
            for b in (a + 1)..adj.len() {
                let n1 = &face_normals[adj[a]];
                let n2 = &face_normals[adj[b]];
                let dot: f64 = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
                if dot < cos_thresh {
                    is_sharp = true;
                    break 'outer;
                }
            }
        }
        if is_sharp {
            let idx: i64 = out.points.len() as i64;
            out.points.push(input.points.get(i));
            verts.push_cell(&[idx]);
        }
    }

    out.verts = verts;
    out
}

/// Compute the normal of a polygon given its vertex indices.
fn polygon_normal(pd: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;

    for i in 0..cell.len() {
        let current = pd.points.get(cell[i] as usize);
        let next = pd.points.get(cell[(i + 1) % cell.len()] as usize);
        nx += (current[1] - next[1]) * (current[2] + next[2]);
        ny += (current[2] - next[2]) * (current[0] + next[0]);
        nz += (current[0] - next[0]) * (current[1] + next[1]);
    }

    let mag: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
    if mag < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [nx / mag, ny / mag, nz / mag]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a simple mesh with a sharp 90-degree fold.
    fn make_fold_mesh() -> PolyData {
        let mut pd = PolyData::new();
        // Two triangles sharing an edge, at 90 degrees to each other
        // Triangle 1 lies in XY plane: (0,0,0), (1,0,0), (0.5,1,0)
        // Triangle 2 folds up: (0,0,0), (1,0,0), (0.5,0,1)
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]); // XY plane triangle
        polys.push_cell(&[0, 1, 3]); // XZ plane triangle (folded up)
        pd.polys = polys;
        pd
    }

    #[test]
    fn sharp_edges_finds_fold() {
        let input = make_fold_mesh();
        // The dihedral angle at edge (0,1) is 90 degrees
        // With threshold 60 degrees, the shared edge should be detected
        let result = extract_sharp_edges(&input, 60.0);
        assert!(
            result.lines.num_cells() > 0,
            "should detect sharp edge at the fold"
        );
    }

    #[test]
    fn sharp_vertices_at_fold() {
        let input = make_fold_mesh();
        // Vertices 0 and 1 are on the fold edge, shared between the two angled faces
        // With a threshold of 60 degrees, they should be detected as sharp
        let result = extract_sharp_vertices(&input, 60.0);
        assert!(
            result.verts.num_cells() >= 2,
            "should detect at least the 2 fold-edge vertices as sharp, got {}",
            result.verts.num_cells()
        );
    }

    #[test]
    fn polygon_normal_uses_all_polygon_vertices() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);

        let normal = polygon_normal(&pd, &[0, 1, 2, 3]);
        assert!(normal[2] > 0.99, "normal was {normal:?}");
    }
}
