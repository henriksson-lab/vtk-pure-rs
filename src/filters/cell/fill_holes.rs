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

    // Build a compact edge list and sort by undirected endpoints. This mirrors
    // VTK's boundary criterion ("no edge neighbor") without allocating a Vec
    // for every point in the mesh.
    let np = input.points.len();
    let boundary_edges = collect_boundary_edges(offsets, conn, nc, np);

    if boundary_edges.is_empty() {
        return input.clone();
    }

    // VTK stores free edges as line cells, builds links, and walks from one
    // line to the single neighboring line at the current endpoint. Use
    // undirected edge adjacency here so reversed boundary-edge orientation
    // does not falsely invalidate a valid loop.
    let sentinel = usize::MAX;
    let mut first_edge_by_vertex = vec![sentinel; np];
    let mut second_edge_by_vertex = vec![sentinel; np];
    let mut edge_degree_by_vertex = vec![0u8; np];
    for (edge_id, &(a, b)) in boundary_edges.iter().enumerate() {
        add_boundary_edge_id(
            a,
            edge_id,
            &mut first_edge_by_vertex,
            &mut second_edge_by_vertex,
            &mut edge_degree_by_vertex,
        );
        add_boundary_edge_id(
            b,
            edge_id,
            &mut first_edge_by_vertex,
            &mut second_edge_by_vertex,
            &mut edge_degree_by_vertex,
        );
    }

    let mut visited = vec![false; boundary_edges.len()];
    let mut output_cells: Option<(Vec<i64>, Vec<i64>)> = None;

    for (start_edge, &(start_v, next_v)) in boundary_edges.iter().enumerate() {
        if visited[start_edge] {
            continue;
        }
        let mut loop_pts = vec![start_v];
        let mut current = next_v;
        let mut current_edge = start_edge;
        let mut valid = true;
        loop {
            visited[current_edge] = true;
            if current == start_v {
                break;
            }
            loop_pts.push(current);

            let mut next_edge = sentinel;
            let mut unvisited_count = 0usize;
            let first = first_edge_by_vertex[current];
            if first != sentinel && !visited[first] {
                next_edge = first;
                unvisited_count += 1;
            }
            let second = second_edge_by_vertex[current];
            if second != sentinel && !visited[second] {
                next_edge = second;
                unvisited_count += 1;
            }
            if edge_degree_by_vertex[current] != 2 || unvisited_count != 1 {
                valid = false;
                break;
            }
            current_edge = next_edge;
            let (a, b) = boundary_edges[current_edge];
            current = if a == current { b } else { a };
        }
        if valid
            && loop_pts.len() >= 3
            && loop_bounding_sphere_radius(input, &loop_pts) <= hole_size
        {
            let (offsets, conn) = output_cells.get_or_insert_with(|| {
                (
                    input.polys.offsets().to_vec(),
                    input.polys.connectivity().to_vec(),
                )
            });
            let added_tris = loop_pts.len() - 2;
            offsets.reserve(added_tris);
            conn.reserve(added_tris * 3);
            for i in 1..loop_pts.len() - 1 {
                conn.extend_from_slice(&[
                    loop_pts[0] as i64,
                    loop_pts[i] as i64,
                    loop_pts[i + 1] as i64,
                ]);
                offsets.push(conn.len() as i64);
            }
        }
    }

    let mut pd = input.clone();
    if let Some((offsets, conn)) = output_cells {
        pd.polys = CellArray::from_raw(offsets, conn);
    }
    pd.cell_data_mut().clear();

    pd
}

fn collect_boundary_edges(
    offsets: &[i64],
    conn: &[i64],
    nc: usize,
    np: usize,
) -> Vec<(usize, usize)> {
    if u32::try_from(np).is_ok() {
        collect_boundary_edges_packed(offsets, conn, nc, np)
    } else {
        collect_boundary_edges_wide(offsets, conn, nc, np)
    }
}

fn collect_boundary_edges_packed(
    offsets: &[i64],
    conn: &[i64],
    nc: usize,
    np: usize,
) -> Vec<(usize, usize)> {
    const EMPTY_KEY: u64 = u64::MAX;

    #[derive(Clone, Copy)]
    struct EdgeSlot {
        key: u64,
        dir: u64,
        count: u8,
    }

    let cap = conn
        .len()
        .saturating_add(conn.len() / 4)
        .saturating_add(1)
        .max(16)
        .next_power_of_two();
    let mut slots = vec![
        EdgeSlot {
            key: EMPTY_KEY,
            dir: 0,
            count: 0,
        };
        cap
    ];
    let mask = cap - 1;

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
            let a = a as u32;
            let b = b as u32;
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            let key = ((lo as u64) << 32) | hi as u64;
            let dir = ((a as u64) << 32) | b as u64;
            let mut idx = edge_hash(key) & mask;
            loop {
                let slot = &mut slots[idx];
                if slot.key == EMPTY_KEY {
                    *slot = EdgeSlot { key, dir, count: 1 };
                    break;
                }
                if slot.key == key {
                    slot.count = slot.count.saturating_add(1);
                    break;
                }
                idx = (idx + 1) & mask;
            }
        }
    }

    let mut boundary_edges = Vec::new();
    for slot in slots {
        if slot.count == 1 {
            boundary_edges.push(((slot.dir >> 32) as usize, (slot.dir & 0xffff_ffff) as usize));
        }
    }
    boundary_edges
}

fn edge_hash(key: u64) -> usize {
    let mut x = key;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    (x ^ (x >> 33)) as usize
}

fn collect_boundary_edges_wide(
    offsets: &[i64],
    conn: &[i64],
    nc: usize,
    np: usize,
) -> Vec<(usize, usize)> {
    let mut edges = Vec::with_capacity(conn.len());
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
            edges.push((lo, hi, a, b));
        }
    }

    edges.sort_unstable_by_key(|&(lo, hi, _, _)| (lo, hi));
    let mut boundary_edges = Vec::new();
    let mut i = 0;
    while i < edges.len() {
        let lo = edges[i].0;
        let hi = edges[i].1;
        let start_i = i;
        i += 1;
        while i < edges.len() && edges[i].0 == lo && edges[i].1 == hi {
            i += 1;
        }
        if i - start_i == 1 {
            let (_, _, a, b) = edges[start_i];
            boundary_edges.push((a, b));
        }
    }
    boundary_edges
}

fn add_boundary_edge_id(
    vertex: usize,
    edge_id: usize,
    first_edge_by_vertex: &mut [usize],
    second_edge_by_vertex: &mut [usize],
    edge_degree_by_vertex: &mut [u8],
) {
    let degree = &mut edge_degree_by_vertex[vertex];
    if *degree == 0 {
        first_edge_by_vertex[vertex] = edge_id;
    } else if *degree == 1 {
        second_edge_by_vertex[vertex] = edge_id;
    }
    *degree = degree.saturating_add(1);
}

fn loop_bounding_sphere_radius(input: &PolyData, loop_pts: &[usize]) -> f64 {
    if loop_pts.is_empty() {
        return 0.0;
    }

    // vtkFillHolesFilter calls vtkSphere::ComputeBoundingSphere with hints
    // initialized to [0, 0], so the sphere starts at the first loop point and
    // grows in a single pass to include subsequent points.
    let first = input.points.get(loop_pts[0]);
    let mut sphere = [first[0], first[1], first[2], 0.0f64];
    let mut radius2 = 0.0f64;
    for &pid in &loop_pts[1..] {
        let p = input.points.get(pid);
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
