use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, VecDeque};

/// Label connected regions of a PolyData mesh by shared edges.
///
/// Two cells are edge-connected if they share an edge (two vertices in common).
/// BFS on edge adjacency assigns a "RegionId" integer to each cell.
/// Returns a clone of the input with a "RegionId" cell data array.
pub fn edge_connectivity(input: &PolyData) -> PolyData {
    let n_cells = input.polys.num_cells();
    if n_cells == 0 {
        return input.clone();
    }

    // Build edge-to-cell map. An edge is identified by (min_id, max_id).
    let mut edge_to_cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();

    for ci in 0..n_cells {
        let cell = input.polys.cell(ci);
        let len = cell.len();
        for e in 0..len {
            let a = cell[e];
            let b = cell[(e + 1) % len];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_cells.entry(key).or_default().push(ci);
        }
    }

    // Build cell adjacency from shared edges
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_cells];
    for (_edge, cells) in &edge_to_cells {
        for i in 0..cells.len() {
            for j in (i + 1)..cells.len() {
                adj[cells[i]].push(cells[j]);
                adj[cells[j]].push(cells[i]);
            }
        }
    }

    // BFS to assign region IDs
    let mut region_ids = vec![-1i32; n_cells];
    let mut current_region = 0i32;

    for start in 0..n_cells {
        if region_ids[start] >= 0 {
            continue;
        }
        let mut queue = VecDeque::new();
        queue.push_back(start);
        region_ids[start] = current_region;

        while let Some(ci) = queue.pop_front() {
            for &neighbor in &adj[ci] {
                if region_ids[neighbor] < 0 {
                    region_ids[neighbor] = current_region;
                    queue.push_back(neighbor);
                }
            }
        }
        current_region += 1;
    }

    let region_f64: Vec<f64> = region_ids.iter().map(|&r| r as f64).collect();

    let mut output = input.clone();
    output
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionId", region_f64, 1,
        )));
    output
}

/// Returns the number of distinct regions found.
pub fn num_edge_regions(input: &PolyData) -> usize {
    let result = edge_connectivity(input);
    if let Some(arr) = result.cell_data().get_array("RegionId") {
        let n = arr.num_tuples();
        let mut buf = [0.0f64];
        let mut max_id = -1.0f64;
        for i in 0..n {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > max_id {
                max_id = buf[0];
            }
        }
        if max_id < 0.0 {
            0
        } else {
            max_id as usize + 1
        }
    } else {
        0
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
                [0.5, 1.0, 0.0],
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let result = edge_connectivity(&pd);
        let arr = result.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        let r0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let r1 = buf[0];

        assert_ne!(r0, r1); // Different regions
        assert_eq!(num_edge_regions(&pd), 2);
    }

    #[test]
    fn shared_edge_same_region() {
        // Two triangles sharing edge (1,2)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let result = edge_connectivity(&pd);
        let arr = result.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        let r0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let r1 = buf[0];

        assert_eq!(r0, r1); // Same region (share edge 1-2)
        assert_eq!(num_edge_regions(&pd), 1);
    }

    #[test]
    fn vertex_only_connection_is_separate() {
        // Two triangles sharing only vertex 1 (not an edge)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );

        // They share vertex 1 but no common edge
        assert_eq!(num_edge_regions(&pd), 2);
    }
}
