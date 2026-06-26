use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Extract the largest connected component from a PolyData mesh.
///
/// Uses union-find on shared vertices to identify connected components,
/// then keeps only cells belonging to the largest group.
pub fn extract_largest_component(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 || input.total_cells() == 0 {
        return PolyData::new();
    }

    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            if cell.len() < 2 {
                continue;
            }
            let first: usize = cell[0] as usize;
            for i in 1..cell.len() {
                union(&mut parent, &mut rank, first, cell[i] as usize);
            }
        }
    }

    let mut component_cell_count: HashMap<usize, usize> = HashMap::new();
    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            if cell.is_empty() {
                continue;
            }
            let root: usize = find(&mut parent, cell[0] as usize);
            *component_cell_count.entry(root).or_insert(0) += 1;
        }
    }

    let largest_root: usize = match component_cell_count.iter().max_by_key(|&(_, &v)| v) {
        Some((&k, _)) => k,
        None => return PolyData::new(),
    };

    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut out_points: Points<f64> = Points::new();

    let out_verts = collect_component_cells(
        &input.verts,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
    );
    let out_lines = collect_component_cells(
        &input.lines,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
    );
    let out_polys = collect_component_cells(
        &input.polys,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
    );
    let out_strips = collect_component_cells(
        &input.strips,
        input,
        &mut parent,
        largest_root,
        &mut point_map,
        &mut out_points,
    );

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.lines = out_lines;
    pd.polys = out_polys;
    pd.strips = out_strips;
    pd
}

fn find(parent: &mut [usize], x: usize) -> usize {
    let mut r: usize = x;
    while parent[r] != r {
        parent[r] = parent[parent[r]];
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

fn collect_component_cells(
    cells: &CellArray,
    input: &PolyData,
    parent: &mut [usize],
    largest_root: usize,
    point_map: &mut HashMap<usize, usize>,
    out_points: &mut Points<f64>,
) -> CellArray {
    let mut output = CellArray::new();
    for cell in cells.iter() {
        if cell.is_empty() {
            continue;
        }
        let root: usize = find(parent, cell[0] as usize);
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
        output.push_cell(&remapped);
    }
    output
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

    #[test]
    fn includes_non_polygon_cells() {
        let mut pd = PolyData::new();
        for i in 0..6 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = extract_largest_component(&pd);
        assert_eq!(result.lines.num_cells(), 2);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 3);
    }
}
