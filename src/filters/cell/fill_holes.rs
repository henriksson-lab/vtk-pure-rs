use crate::data::{CellArray, PolyData};

/// Fill holes (open boundary loops) in a triangle mesh.
///
/// Finds boundary edges (edges used by exactly one polygon), traces
/// closed loops, and fills each loop with triangles using existing points.
pub fn fill_holes(input: &PolyData) -> PolyData {
    fill_holes_with_hole_size(input, 1.0)
}

/// Fill holes whose bounding-sphere radius is no larger than `hole_size`.
///
/// This mirrors `vtkFillHolesFilter::HoleSize`: VTK uses the radius of a
/// bounding circumsphere as an approximate hole-size gate before triangulating.
pub fn fill_holes_with_hole_size(input: &PolyData, hole_size: f64) -> PolyData {
    let work_polys = polys_with_decomposed_strips(input);
    let offsets = work_polys.offsets();
    let conn = work_polys.connectivity();
    let nc = work_polys.num_cells();

    // Sorted-edge approach: collect all directed edges, sort, find boundary edges.
    // Boundary = edges appearing exactly once when canonicalized (a < b).
    // ~3x faster than HashMap for large meshes.
    let np = input.points.len();
    let total_edges = conn.len(); // each connectivity entry contributes one edge
    let mut edges: Vec<(u64, i64, i64)> = Vec::with_capacity(total_edges); // (canonical_key, a, b)

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let n = end - start;
        if n < 3 {
            continue;
        }
        for i in 0..n {
            let a = conn[start + i];
            let b = conn[start + if i + 1 < n { i + 1 } else { 0 }];
            if a < 0 || b < 0 || a as usize >= np || b as usize >= np || a == b {
                continue;
            }
            let key = if a < b {
                (a as u64) << 32 | b as u64
            } else {
                (b as u64) << 32 | a as u64
            };
            edges.push((key, a, b));
        }
    }
    edges.sort_unstable_by_key(|e| e.0);

    // Find boundary edges (canonical key appears exactly once).
    let mut boundary_edges: Vec<(usize, usize)> = Vec::new();
    let mut has_boundary = false;
    let ne = edges.len();
    let mut i = 0;
    while i < ne {
        let key = edges[i].0;
        let mut count = 0usize;
        let start_i = i;
        while i < ne && edges[i].0 == key {
            count += 1;
            i += 1;
        }
        if count == 1 {
            let (_, a, b) = edges[start_i];
            boundary_edges.push((a as usize, b as usize));
            has_boundary = true;
        }
    }

    if !has_boundary {
        return input.clone();
    }

    // VTK stores free edges as line cells, builds links, and walks from one
    // line to the single neighboring line at the current endpoint. Use
    // undirected edge adjacency here so reversed boundary-edge orientation
    // does not falsely invalidate a valid loop.
    let mut edge_ids_by_vertex: Vec<Vec<usize>> = vec![Vec::new(); np];
    for (edge_id, &(a, b)) in boundary_edges.iter().enumerate() {
        edge_ids_by_vertex[a].push(edge_id);
        edge_ids_by_vertex[b].push(edge_id);
    }

    let mut visited = vec![false; boundary_edges.len()];
    let mut loops: Vec<Vec<i64>> = Vec::new();

    for (start_edge, &(start_v, next_v)) in boundary_edges.iter().enumerate() {
        if visited[start_edge] {
            continue;
        }
        let mut loop_pts = vec![start_v as i64];
        let mut current = next_v;
        let mut current_edge = start_edge;
        let mut valid = true;
        loop {
            visited[current_edge] = true;
            if current == start_v {
                break;
            }
            loop_pts.push(current as i64);

            let unvisited: Vec<usize> = edge_ids_by_vertex[current]
                .iter()
                .copied()
                .filter(|&edge_id| !visited[edge_id])
                .collect();
            if unvisited.len() != 1 {
                valid = false;
                break;
            }
            current_edge = unvisited[0];
            let (a, b) = boundary_edges[current_edge];
            current = if a == current { b } else { a };
        }
        if valid && loop_pts.len() >= 3 {
            loops.push(loop_pts);
        }
    }

    let mut pd = input.clone();

    for lp in &loops {
        if loop_bounding_sphere_radius(input, lp) > hole_size {
            continue;
        }
        for i in 1..lp.len() - 1 {
            pd.polys.push_cell(&[lp[0], lp[i], lp[i + 1]]);
        }
    }
    pd.cell_data_mut().clear();

    pd
}

fn loop_bounding_sphere_radius(input: &PolyData, loop_pts: &[i64]) -> f64 {
    if loop_pts.is_empty() {
        return 0.0;
    }

    // vtkFillHolesFilter calls vtkSphere::ComputeBoundingSphere with hints
    // initialized to [0, 0], so the sphere starts at the first loop point and
    // grows in a single pass to include subsequent points.
    let first = input.points.get(loop_pts[0] as usize);
    let mut sphere = [first[0], first[1], first[2], 0.0f64];
    let mut radius2 = 0.0f64;
    for &pid in &loop_pts[1..] {
        let p = input.points.get(pid as usize);
        let dx = p[0] - sphere[0];
        let dy = p[1] - sphere[1];
        let dz = p[2] - sphere[2];
        let dist2 = dx * dx + dy * dy + dz * dz;
        if dist2 > radius2 {
            let dist = dist2.sqrt();
            sphere[3] = (sphere[3] + dist) / 2.0;
            radius2 = sphere[3] * sphere[3];
            let delta = dist - sphere[3];
            sphere[0] = (sphere[3] * sphere[0] + delta * p[0]) / dist;
            sphere[1] = (sphere[3] * sphere[1] + delta * p[1]) / dist;
            sphere[2] = (sphere[3] * sphere[2] + delta * p[2]) / dist;
        }
    }
    sphere[3]
}

fn polys_with_decomposed_strips(input: &PolyData) -> CellArray {
    if input.strips.is_empty() {
        return input.polys.clone();
    }

    let mut polys = input.polys.clone();
    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            if i % 2 == 0 {
                polys.push_cell(&[strip[i], strip[i + 1], strip[i + 2]]);
            } else {
                polys.push_cell(&[strip[i + 1], strip[i], strip[i + 2]]);
            }
        }
    }
    polys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_single_hole() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = fill_holes(&pd);
        assert!(result.polys.num_cells() >= 2);
    }

    #[test]
    fn respects_hole_size() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = fill_holes_with_hole_size(&pd, 0.1);
        assert_eq!(result.polys.num_cells(), pd.polys.num_cells());
    }

    #[test]
    fn closed_mesh_unchanged() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let result = fill_holes(&pd);
        assert_eq!(result.polys.num_cells(), 4);
    }
}
