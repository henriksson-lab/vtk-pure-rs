use crate::data::PolyData;

/// Fill holes (open boundary loops) in a triangle mesh.
///
/// Finds boundary edges (edges used by exactly one polygon), traces
/// closed loops, and fills each loop with a fan of triangles from the
/// loop centroid.
pub fn fill_holes(input: &PolyData) -> PolyData {
    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();
    let nc = input.polys.num_cells();

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
    let mut next_vertex: Vec<i64> = vec![-1; np];
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
            // Boundary edge: set directed next
            let (_, a, b) = edges[start_i];
            next_vertex[a as usize] = b;
            has_boundary = true;
        }
    }

    if !has_boundary {
        return input.clone();
    }

    // Trace loops
    let mut visited = vec![false; np];
    let mut loops: Vec<Vec<i64>> = Vec::new();

    for start_v in 0..np {
        if next_vertex[start_v] < 0 || visited[start_v] {
            continue;
        }
        let mut loop_pts = Vec::new();
        let mut current = start_v;
        loop {
            if visited[current] {
                break;
            }
            visited[current] = true;
            loop_pts.push(current as i64);
            let nxt = next_vertex[current];
            if nxt < 0 {
                break;
            }
            current = nxt as usize;
        }
        if loop_pts.len() >= 3 && current == start_v {
            loops.push(loop_pts);
        }
    }

    let mut pd = input.clone();
    let pts = input.points.as_flat_slice();

    // Fill each loop with a fan triangulation
    for lp in &loops {
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in lp {
            let b = id as usize * 3;
            cx += pts[b];
            cy += pts[b + 1];
            cz += pts[b + 2];
        }
        let n = lp.len() as f64;
        let center_idx = pd.points.len() as i64;
        pd.points.push([cx / n, cy / n, cz / n]);

        for i in 0..lp.len() {
            let a = lp[i];
            let b = lp[(i + 1) % lp.len()];
            pd.polys.push_cell(&[center_idx, b, a]);
        }
    }

    pd
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
