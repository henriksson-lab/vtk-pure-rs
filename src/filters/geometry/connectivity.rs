use crate::data::{CellArray, Points, PolyData};

/// Extract connected components from a PolyData mesh.
///
/// Returns a Vec of PolyData, one per connected component, sorted by size (largest first).
pub fn extract_components(input: &PolyData) -> Vec<PolyData> {
    let n = input.points.len();
    if n == 0 {
        return Vec::new();
    }

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0usize; n];

    // Union all points in each cell using raw connectivity for speed
    let all_cells: [&crate::data::CellArray; 4] =
        [&input.polys, &input.verts, &input.lines, &input.strips];
    for cells in &all_cells {
        let off = cells.offsets();
        let con = cells.connectivity();
        let nc = cells.num_cells();
        for ci in 0..nc {
            let start = off[ci] as usize;
            let end = off[ci + 1] as usize;
            if end - start < 2 {
                continue;
            }
            let first = con[start] as usize;
            for idx in (start + 1)..end {
                union(&mut parent, &mut rank, first, con[idx] as usize);
            }
        }
    }

    // Group cell indices by root (avoids copying cell data)
    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();
    let nc = input.polys.num_cells();

    let mut component_cells: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        if start >= end {
            continue;
        }
        let root = find(&mut parent, conn[start] as usize);
        component_cells.entry(root).or_default().push(ci);
    }

    // Sort by size (largest first)
    let mut components: Vec<Vec<usize>> = component_cells.into_values().collect();
    components.sort_by(|a, b| b.len().cmp(&a.len()));

    // Build output PolyData for each component using flat slice access
    let pts = input.points.as_flat_slice();
    components
        .into_iter()
        .map(|cell_indices| build_component_fast(pts, offsets, conn, n, &cell_indices))
        .collect()
}

/// Extract only the largest connected component.
pub fn extract_largest_component(input: &PolyData) -> PolyData {
    let mut components = extract_components(input);
    if components.is_empty() {
        PolyData::new()
    } else {
        components.remove(0)
    }
}

fn build_component_fast(
    pts: &[f64],
    offsets: &[i64],
    conn: &[i64],
    n_pts: usize,
    cell_indices: &[usize],
) -> PolyData {
    // Use flat Vec for point remapping (avoids HashMap)
    let mut pt_map: Vec<i64> = vec![-1; n_pts];
    let mut pts_flat: Vec<f64> = Vec::new();
    let mut out_off: Vec<i64> = Vec::with_capacity(cell_indices.len() + 1);
    let mut out_conn: Vec<i64> = Vec::new();
    out_off.push(0);

    for &ci in cell_indices {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        for idx in start..end {
            let old_id = conn[idx] as usize;
            if pt_map[old_id] < 0 {
                pt_map[old_id] = (pts_flat.len() / 3) as i64;
                let b = old_id * 3;
                pts_flat.extend_from_slice(&pts[b..b + 3]);
            }
            out_conn.push(pt_map[old_id]);
        }
        out_off.push(out_conn.len() as i64);
    }

    let mut output = PolyData::new();
    output.points = Points::from_flat_vec(pts_flat);
    output.polys = CellArray::from_raw(out_off, out_conn);
    output
}

fn find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = find(parent, parent[x]);
    }
    parent[x]
}

fn union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        parent[ra] = rb;
    } else if rank[ra] > rank[rb] {
        parent[rb] = ra;
    } else {
        parent[rb] = ra;
        rank[ra] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_disconnected_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                // Separate triangle
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let components = extract_components(&pd);
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].polys.num_cells(), 1);
        assert_eq!(components[1].polys.num_cells(), 1);
    }

    #[test]
    fn single_connected_mesh() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let components = extract_components(&pd);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].polys.num_cells(), 2);
    }

    #[test]
    fn largest_component() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                // Separate single triangle
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [4, 5, 6]],
        );

        let largest = extract_largest_component(&pd);
        assert_eq!(largest.polys.num_cells(), 2);
    }
}
