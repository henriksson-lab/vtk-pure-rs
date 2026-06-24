use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extract the largest connected component from a triangle mesh.
///
/// Uses union-find on shared vertices to identify connected components,
/// then keeps only cells belonging to the largest group.
pub fn extract_largest_component(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    // Union-Find
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    fn find(parent: &mut [usize], x: usize) -> usize {
        let mut r: usize = x;
        while parent[r] != r {
            parent[r] = parent[parent[r]]; // path compression
            r = parent[r];
        }
        r
    }

    fn union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
        let ra: usize = find(parent, a);
        let rb: usize = find(parent, b);
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

    // Union vertices that share an edge (belong to the same cell)
    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        let first: usize = cell[0] as usize;
        for i in 1..cell.len() {
            union(&mut parent, &mut rank, first, cell[i] as usize);
        }
    }

    // Count cells per component (by root)
    let mut component_cell_count: HashMap<usize, usize> = HashMap::new();
    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let root: usize = find(&mut parent, cell[0] as usize);
        *component_cell_count.entry(root).or_insert(0) += 1;
    }

    // Find the largest component
    let largest_root: usize = match component_cell_count.iter().max_by_key(|&(_, &v)| v) {
        Some((&k, _)) => k,
        None => return PolyData::new(),
    };

    // Collect cells that belong to the largest component, remap points
    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut out_points: Points<f64> = Points::new();
    let mut out_polys: CellArray = CellArray::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let root: usize = find(&mut parent, cell[0] as usize);
        if root != largest_root {
            continue;
        }
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&id| {
                let idx: usize = id as usize;
                let next_id: usize = out_points.len();
                *point_map.entry(idx).or_insert_with(|| {
                    out_points.push(input.points.get(idx));
                    next_id
                }) as i64
            })
            .collect();
        out_polys.push_cell(&remapped);
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
    fn single_component() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        // Points 1 and 3 are the same location but different indices;
        // however they share no cell, so if there were two components we'd see fewer cells.
        // Actually both triangles share point index 3 which == point 1 position but
        // we test single-connected-component by sharing vertex indices.
        let pd2 = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extract_largest_component(&pd2);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn two_components_picks_larger() {
        // Component A: 2 triangles sharing edge 1-2
        // Component B: 1 triangle (disconnected vertices)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],    // 0 - comp A
                [1.0, 0.0, 0.0],    // 1 - comp A
                [0.5, 1.0, 0.0],    // 2 - comp A
                [1.5, 1.0, 0.0],    // 3 - comp A
                [10.0, 10.0, 10.0], // 4 - comp B
                [11.0, 10.0, 10.0], // 5 - comp B
                [10.5, 11.0, 10.0], // 6 - comp B
            ],
            vec![[0, 1, 2], [1, 2, 3], [4, 5, 6]],
        );
        let result = extract_largest_component(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = extract_largest_component(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }
}
