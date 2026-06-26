use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Sqrt(3) subdivision.
///
/// Inserts a vertex at each triangle centroid, connects centroids
/// of adjacent triangles, then flips the old edges. Produces a
/// finer mesh with 3x the face count. Smoother than Loop subdivision
/// for some applications.
pub fn subdivide_sqrt3(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let tris: Vec<[i64; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    if tris.is_empty() {
        return PolyData::new();
    }

    let mut out_pts = Points::<f64>::new();
    for i in 0..n {
        out_pts.push(input.points.get(i));
    }

    // Insert centroids
    let mut centroid_ids: Vec<i64> = Vec::with_capacity(tris.len());
    for tri in &tris {
        let v0 = input.points.get(tri[0] as usize);
        let v1 = input.points.get(tri[1] as usize);
        let v2 = input.points.get(tri[2] as usize);
        let idx = out_pts.len() as i64;
        out_pts.push([
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ]);
        centroid_ids.push(idx);
    }

    // Build edge-face adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut out_polys = CellArray::new();

    // For each original edge shared by two triangles, create 2 new triangles
    // connecting the centroids to the edge endpoints
    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 2 {
            let c0 = centroid_ids[faces[0]];
            let c1 = centroid_ids[faces[1]];
            out_polys.push_cell(&[a, c0, c1]);
            out_polys.push_cell(&[b, c1, c0]);
        } else if faces.len() == 1 {
            // Boundary edge: connect to single centroid
            let c = centroid_ids[faces[0]];
            out_polys.push_cell(&[a, b, c]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivide_single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_sqrt3(&pd);
        assert_eq!(result.points.len(), 4); // 3 + 1 centroid
        assert_eq!(result.polys.num_cells(), 3); // 3 boundary edges -> 3 tris
    }

    #[test]
    fn subdivide_two_tris() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = subdivide_sqrt3(&pd);
        assert_eq!(result.points.len(), 6); // 4 + 2 centroids
        assert!(result.polys.num_cells() > 4);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = subdivide_sqrt3(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
