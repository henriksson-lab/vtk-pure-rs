use std::collections::{HashMap, HashSet};

use crate::data::{CellArray, Points, PolyData};

/// Merge coplanar adjacent triangles into larger polygons.
///
/// Two triangles sharing an edge are merged if the angle between their normals
/// is less than `angle_tolerance_deg` degrees.  The shared edge is removed and
/// the two triangles are combined into a single polygon.
///
/// Only triangles (3-vertex cells) are considered for merging.  Non-triangle
/// polygons are passed through unchanged.
pub fn merge_coplanar_faces(input: &PolyData, angle_tolerance_deg: f64) -> PolyData {
    let cos_tol: f64 = angle_tolerance_deg.to_radians().cos();

    // Collect all polygon cells as vectors of point ids.
    let mut cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let cell_count: usize = cells.len();

    // Compute face normals (Newell's method).
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| polygon_normal(&input.points, c))
        .collect();

    // Build edge -> face index mapping (only for triangles).
    // Key: sorted (min, max) point id pair.
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        if cell.len() != 3 {
            continue;
        }
        let n: usize = cell.len();
        for i in 0..n {
            let a: i64 = cell[i];
            let b: i64 = cell[(i + 1) % n];
            let key: (i64, i64) = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Union-find for merging faces.
    let mut parent: Vec<usize> = (0..cell_count).collect();

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        let mut r: usize = x;
        while parent[r] != r {
            parent[r] = parent[parent[r]];
            r = parent[r];
        }
        r
    }

    fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
        let ra: usize = find(parent, a);
        let rb: usize = find(parent, b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    // Track which edges are shared between merged faces so we can remove them.
    let mut shared_edges: HashSet<(i64, i64)> = HashSet::new();

    for (&edge, faces) in &edge_faces {
        if faces.len() == 2 {
            let fi: usize = faces[0];
            let fj: usize = faces[1];
            // Both must be triangles.
            if cells[fi].len() != 3 || cells[fj].len() != 3 {
                continue;
            }
            let n1: [f64; 3] = normals[fi];
            let n2: [f64; 3] = normals[fj];
            let dot: f64 = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
            if dot >= cos_tol {
                union(&mut parent, fi, fj);
                shared_edges.insert(edge);
            }
        }
    }

    // Group faces by their root.
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..cell_count {
        let r: usize = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }

    // Build output.
    let mut out_points = input.points.clone();
    let _ = &out_points; // keep
    let mut out_polys = CellArray::new();

    for (_root, face_ids) in &groups {
        if face_ids.len() == 1 {
            // Single face, pass through.
            out_polys.push_cell(&cells[face_ids[0]]);
        } else {
            // Merge: collect all edges that are NOT shared, forming the
            // boundary loop of the merged polygon.
            let mut boundary_edges: Vec<(i64, i64)> = Vec::new();
            for &fi in face_ids {
                let c: &Vec<i64> = &cells[fi];
                let n: usize = c.len();
                for i in 0..n {
                    let a: i64 = c[i];
                    let b: i64 = c[(i + 1) % n];
                    let key: (i64, i64) = if a < b { (a, b) } else { (b, a) };
                    if !shared_edges.contains(&key) {
                        boundary_edges.push((a, b)); // directed edge
                    }
                }
            }

            // Chain the boundary edges into a polygon loop.
            if boundary_edges.is_empty() {
                // Shouldn't happen, but pass the first face through.
                out_polys.push_cell(&cells[face_ids[0]]);
                continue;
            }

            // Build adjacency from start -> end for directed edges.
            let mut next_map: HashMap<i64, i64> = HashMap::new();
            for &(a, b) in &boundary_edges {
                next_map.insert(a, b);
            }

            let start: i64 = boundary_edges[0].0;
            let mut loop_pts: Vec<i64> = Vec::new();
            let mut cur: i64 = start;
            for _ in 0..boundary_edges.len() + 1 {
                loop_pts.push(cur);
                match next_map.get(&cur) {
                    Some(&nxt) => {
                        if nxt == start {
                            break;
                        }
                        cur = nxt;
                    }
                    None => break,
                }
            }

            if loop_pts.len() >= 3 {
                out_polys.push_cell(&loop_pts);
            } else {
                // Fallback: emit individual faces.
                for &fi in face_ids {
                    out_polys.push_cell(&cells[fi]);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn polygon_normal(points: &Points<f64>, cell: &[i64]) -> [f64; 3] {
    let mut nx: f64 = 0.0;
    let mut ny: f64 = 0.0;
    let mut nz: f64 = 0.0;
    let n: usize = cell.len();
    if n < 3 {
        return [0.0, 0.0, 1.0];
    }
    for i in 0..n {
        let p: [f64; 3] = points.get(cell[i] as usize);
        let q: [f64; 3] = points.get(cell[(i + 1) % n] as usize);
        nx += (p[1] - q[1]) * (p[2] + q[2]);
        ny += (p[2] - q[2]) * (p[0] + q[0]);
        nz += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-20 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_coplanar_triangles_merge() {
        // Two coplanar triangles sharing edge 1-2, forming a quad.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let result = merge_coplanar_faces(&pd, 1.0);
        // Should merge into a single polygon.
        assert_eq!(result.polys.num_cells(), 1);
        // The merged polygon should have 4 vertices.
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4);
    }

    #[test]
    fn non_coplanar_stay_separate() {
        // Two triangles at 90 degrees: one in XY plane, one in XZ plane.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let result = merge_coplanar_faces(&pd, 5.0);
        // Should remain two separate triangles.
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn single_triangle_unchanged() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = merge_coplanar_faces(&pd, 10.0);
        assert_eq!(result.polys.num_cells(), 1);
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 3);
    }
}
