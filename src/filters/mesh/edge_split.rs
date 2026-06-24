use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Split edges at sharp features while keeping the mesh watertight.
///
/// Duplicates vertices along sharp edges (dihedral angle > threshold)
/// so that each side of the edge has its own vertex copy. This enables
/// flat shading across sharp edges while keeping smooth shading elsewhere.
pub fn split_sharp_edges(input: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let cos_thresh = angle_threshold_deg.to_radians().cos();

    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0; 3];
            }
            let v0 = input.points.get(c[0] as usize);
            let v1 = input.points.get(c[1] as usize);
            let v2 = input.points.get(c[2] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                [n[0] / l, n[1] / l, n[2] / l]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    // Find sharp edges
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut sharp_edges: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();
    for (&edge, faces) in &edge_faces {
        if faces.len() == 2 {
            let na = normals[faces[0]];
            let nb = normals[faces[1]];
            let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
            if dot < cos_thresh {
                sharp_edges.insert(edge);
            }
        }
    }

    // Duplicate vertices: each face gets its own copy of vertices on sharp edges
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut pt_per_face: HashMap<(usize, i64), i64> = HashMap::new();

    for (fi, c) in cells.iter().enumerate() {
        let mut ids = Vec::with_capacity(c.len());
        for &vid in c {
            // Check if this vertex is on a sharp edge for this face
            let on_sharp = c.iter().any(|&other| {
                if other == vid {
                    return false;
                }
                let key = if vid < other {
                    (vid, other)
                } else {
                    (other, vid)
                };
                sharp_edges.contains(&key)
            });

            let new_id = if on_sharp {
                // Per-face copy
                let key = (fi, vid);
                *pt_per_face.entry(key).or_insert_with(|| {
                    let idx = out_points.len() as i64;
                    out_points.push(input.points.get(vid as usize));
                    idx
                })
            } else {
                // Shared
                let key = (usize::MAX, vid);
                *pt_per_face.entry(key).or_insert_with(|| {
                    let idx = out_points.len() as i64;
                    out_points.push(input.points.get(vid as usize));
                    idx
                })
            };
            ids.push(new_id);
        }
        out_polys.push_cell(&ids);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_90_degree() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = split_sharp_edges(&pd, 45.0);
        assert!(result.points.len() > 4); // duplicated
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn flat_no_split() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = split_sharp_edges(&pd, 10.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = split_sharp_edges(&pd, 30.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
