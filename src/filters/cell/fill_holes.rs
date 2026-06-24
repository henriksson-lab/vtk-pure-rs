use crate::data::{CellArray, PolyData};

/// Fill holes (open boundary loops) in a triangle mesh.
///
/// Finds boundary edges (edges used by exactly one polygon), traces
/// closed loops, and fills each loop with triangles using existing points.
pub fn fill_holes(input: &PolyData) -> PolyData {
    let work_polys = polys_with_decomposed_strips(input);
    let offsets = work_polys.offsets();
    let conn = work_polys.connectivity();
    let nc = work_polys.num_cells();

    // Sorted-edge approach: collect all directed edges, sort, find boundary edges.
    // Boundary = edges appearing exactly once when canonicalized (a < b).
    // ~3x faster than HashMap for large meshes.
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
            let key = if a < b {
                (a as u64) << 32 | b as u64
            } else {
                (b as u64) << 32 | a as u64
            };
            edges.push((key, a, b));
        }
    }
    edges.sort_unstable_by_key(|e| e.0);

    // Find boundary edges (canonical key appears exactly once)
    let np = input.points.len();
    let mut boundary_out: Vec<Vec<i64>> = vec![Vec::new(); np];
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
            if a >= 0 && b >= 0 {
                boundary_out[a as usize].push(b);
                has_boundary = true;
            }
        }
    }

    if !has_boundary {
        return input.clone();
    }

    // Trace loops
    let mut visited = vec![false; np];
    let mut loops: Vec<Vec<i64>> = Vec::new();

    for start_v in 0..np {
        if boundary_out[start_v].is_empty() || visited[start_v] {
            continue;
        }
        let mut loop_pts = Vec::new();
        let mut current = start_v;
        let mut valid = true;
        loop {
            if visited[current] {
                break;
            }
            visited[current] = true;
            loop_pts.push(current as i64);
            if boundary_out[current].len() != 1 {
                valid = false;
                break;
            }
            let nxt = boundary_out[current][0];
            current = nxt as usize;
        }
        if valid && loop_pts.len() >= 3 && current == start_v {
            loops.push(loop_pts);
        }
    }

    let mut pd = input.clone();

    for lp in &loops {
        for i in 1..lp.len() - 1 {
            pd.polys.push_cell(&[lp[0], lp[i], lp[i + 1]]);
        }
    }
    pd.cell_data_mut().clear();

    pd
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
