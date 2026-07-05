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

    // Bucket edges by their lower endpoint, then sort only each small point
    // bucket. This is the same boundary criterion as VTK's "no edge neighbor"
    // check, but avoids a global sort across every mesh edge.
    let np = input.points.len();
    let mut edge_counts_by_lower = vec![0usize; np];

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
            let (lo, _) = if a < b {
                (a as usize, b as usize)
            } else {
                (b as usize, a as usize)
            };
            edge_counts_by_lower[lo] += 1;
        }
    }

    let mut edges_by_lower: Vec<Vec<(usize, usize, usize)>> = edge_counts_by_lower
        .into_iter()
        .map(Vec::with_capacity)
        .collect();

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
            let a = a as usize;
            let b = b as usize;
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            edges_by_lower[lo].push((hi, a, b));
        }
    }

    let mut boundary_edges: Vec<(usize, usize)> = Vec::new();
    let mut has_boundary = false;
    for bucket in &mut edges_by_lower {
        bucket.sort_unstable_by_key(|edge| edge.0);
        let mut i = 0;
        while i < bucket.len() {
            let hi = bucket[i].0;
            let start_i = i;
            i += 1;
            while i < bucket.len() && bucket[i].0 == hi {
                i += 1;
            }
            if i - start_i == 1 {
                let (_, a, b) = bucket[start_i];
                boundary_edges.push((a, b));
                has_boundary = true;
            }
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

            let mut next_edge = None;
            let mut unvisited_count = 0usize;
            for &edge_id in &edge_ids_by_vertex[current] {
                if !visited[edge_id] {
                    next_edge = Some(edge_id);
                    unvisited_count += 1;
                    if unvisited_count > 1 {
                        break;
                    }
                }
            }
            let Some(next_edge) = next_edge else {
                valid = false;
                break;
            };
            if unvisited_count != 1 {
                valid = false;
                break;
            }
            current_edge = next_edge;
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
