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
///
/// The normal test is *signed*, matching `vtkGenerateRegionIds`
/// (`Dot(n0, n1) > cos(MaxAngle)`): faces whose normals oppose each other are
/// never merged, even though they lie in a common plane.  Re-orienting an
/// inconsistently wound surface is the job of the normals/orient filters, and
/// merging back-to-back faces would silently destroy geometry.
///
/// The merged boundary is recovered by cancelling the directed edges shared
/// inside a group, as `vtkPolygonBuilder` does, so the emitted polygon keeps
/// the winding of the faces it replaces.  A group whose surviving edges do not
/// form exactly one closed loop (open chain, several loops, a hole, or a pinch
/// vertex) is not representable as a single polygon and is emitted unmerged.
pub fn merge_coplanar_faces(input: &PolyData, angle_tolerance_deg: f64) -> PolyData {
    let cos_tol: f64 = angle_tolerance_deg.to_radians().cos();
    let num_points: usize = input.points.len();

    // Collect all polygon cells as vectors of point ids.
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
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
            if valid_point_id(a, num_points).is_none() || valid_point_id(b, num_points).is_none() {
                continue;
            }
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

    // Group faces by their root, preserving first-cell order in the output so
    // that the result does not depend on hash iteration order.
    let mut group_index: HashMap<usize, usize> = HashMap::new();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for i in 0..cell_count {
        let r: usize = find(&mut parent, i);
        let gi = *group_index.entry(r).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[gi].push(i);
    }

    // Build output.
    let out_points = input.points.clone();
    let mut out_polys = CellArray::new();

    for face_ids in &groups {
        if face_ids.len() == 1 {
            // Single face, pass through.
            out_polys.push_cell(&cells[face_ids[0]]);
            continue;
        }

        // Merge: collect the directed edges that are NOT shared.  Interior
        // edges cancel, leaving the boundary loop of the merged polygon.
        let mut boundary_edges: Vec<(i64, i64)> = Vec::new();
        for &fi in face_ids {
            let c: &Vec<i64> = &cells[fi];
            let n: usize = c.len();
            for i in 0..n {
                let a: i64 = c[i];
                let b: i64 = c[(i + 1) % n];
                if valid_point_id(a, num_points).is_none()
                    || valid_point_id(b, num_points).is_none()
                {
                    continue;
                }
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

        // Directed adjacency start -> end.  A well-formed patch boundary has
        // exactly one outgoing edge per boundary vertex; anything else is a
        // pinch point and cannot be walked unambiguously.
        let mut next_map: HashMap<i64, i64> = HashMap::new();
        let mut fan_out: bool = false;
        for &(a, b) in &boundary_edges {
            if next_map.insert(a, b).is_some() {
                fan_out = true;
            }
        }

        let mut loop_pts: Vec<i64> = Vec::new();
        if !fan_out {
            let start: i64 = boundary_edges[0].0;
            let mut cur: i64 = start;
            let mut closed: bool = false;
            for _ in 0..boundary_edges.len() {
                loop_pts.push(cur);
                match next_map.get(&cur) {
                    Some(&nxt) => {
                        if nxt == start {
                            closed = true;
                            break;
                        }
                        cur = nxt;
                    }
                    None => break,
                }
            }
            // Accept only a single closed loop that consumed every boundary
            // edge.  An open chain or a second loop (a hole) is not one polygon.
            if !closed || loop_pts.len() != boundary_edges.len() {
                loop_pts.clear();
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
        let Some(pi) = valid_point_id(cell[i], points.len()) else {
            return [0.0, 0.0, 1.0];
        };
        let Some(qi) = valid_point_id(cell[(i + 1) % n], points.len()) else {
            return [0.0, 0.0, 1.0];
        };
        let p: [f64; 3] = points.get(pi);
        let q: [f64; 3] = points.get(qi);
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

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
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

    #[test]
    fn merged_polygon_keeps_source_winding() {
        // The quad replaces two +z-facing triangles, so its own Newell normal
        // must still point along +z.
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
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        let n = polygon_normal(&result.points, &cell);
        assert!(n[2] > 0.9, "merged quad should stay +z wound, got {n:?}");
    }

    #[test]
    fn reversed_winding_triangles_are_not_merged() {
        // [0,1,2] faces +z, [3,2,0] faces -z.  The two lie in a common plane
        // but their normals are anti-parallel, so the signed test rejects them
        // (as vtkGenerateRegionIds does).  Merging them would have to flip one
        // face's orientation, which this filter must not do silently.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 2, 0]],
        );
        let result = merge_coplanar_faces(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn back_to_back_faces_are_both_preserved() {
        // A zero-thickness "fin": the same triangle wound both ways.  An
        // unsigned normal test would merge these, find every edge shared, and
        // silently drop one of the two faces.
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2], [0, 2, 1]],
        );
        let result = merge_coplanar_faces(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn invalid_point_ids_do_not_panic() {
        let pd = PolyData::from_polygons(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![vec![0, 1, 2], vec![-1, 99, 2]],
        );
        let result = merge_coplanar_faces(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn coplanar_fan_merges_into_one_polygon() {
        // Four coplanar triangles fanning around an interior apex tile a unit
        // square; the apex must disappear from the merged boundary.
        let pd = PolyData::from_triangles(
            vec![
                [0.5, 0.5, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 1]],
        );
        let result = merge_coplanar_faces(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 1);
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4);
        assert!(!cell.contains(&0), "interior fan apex must be dropped");
    }

    #[test]
    fn output_cell_order_is_deterministic() {
        // Disjoint, non-mergeable triangles must come out in input order every
        // run; grouping by hash-map iteration order would not guarantee that.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.0, 0.0, 1.0],
                [9.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [9.0, 1.0, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
        );
        let first: Vec<Vec<i64>> = merge_coplanar_faces(&pd, 1.0)
            .polys
            .iter()
            .map(|c| c.to_vec())
            .collect();
        for _ in 0..8 {
            let again: Vec<Vec<i64>> = merge_coplanar_faces(&pd, 1.0)
                .polys
                .iter()
                .map(|c| c.to_vec())
                .collect();
            assert_eq!(first, again);
        }
        assert_eq!(first, vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]]);
    }
}
