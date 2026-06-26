use crate::data::{CellArray, Points, PolyData};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Extract all boundary loops as separate polyline cells.
///
/// Each boundary loop is a closed polyline of edges that belong to
/// exactly one triangle. Returns a PolyData with line cells.
pub fn extract_boundary_loops(input: &PolyData) -> PolyData {
    let boundary_next = boundary_adjacency(input);

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut visited_edges = HashSet::new();
    let mut pt_map: HashMap<i64, i64> = HashMap::new();

    let map_pt =
        |id: i64, pts: &PolyData, out: &mut Points<f64>, map: &mut HashMap<i64, i64>| -> i64 {
            *map.entry(id).or_insert_with(|| {
                let idx = out.len() as i64;
                out.push(pts.points.get(id as usize));
                idx
            })
        };

    for &start in boundary_next.keys() {
        while let Some(&first_next) = boundary_next.get(&start).and_then(|nexts| {
            nexts
                .iter()
                .find(|&&n| !visited_edges.contains(&edge_key(start, n)))
        }) {
            let mut ids = vec![start];
            let mut prev = start;
            let mut cur = start;
            let mut next = first_next;

            loop {
                visited_edges.insert(edge_key(cur, next));
                ids.push(next);

                prev = cur;
                cur = next;
                if cur == start {
                    break;
                }

                let Some(nexts) = boundary_next.get(&cur) else {
                    break;
                };
                let candidate = nexts
                    .iter()
                    .copied()
                    .filter(|&n| n != prev)
                    .find(|&n| !visited_edges.contains(&edge_key(cur, n)))
                    .or_else(|| {
                        nexts
                            .iter()
                            .copied()
                            .find(|&n| !visited_edges.contains(&edge_key(cur, n)))
                    });

                match candidate {
                    Some(n) => next = n,
                    None => break,
                }
            }

            if ids.len() >= 2 {
                let line_ids: Vec<i64> = ids
                    .into_iter()
                    .map(|id| map_pt(id, input, &mut out_points, &mut pt_map))
                    .collect();
                out_lines.push_cell(&line_ids);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

/// Count the number of boundary loops.
pub fn num_boundary_loops(input: &PolyData) -> usize {
    let loops = extract_boundary_loops(input);
    loops.lines.num_cells()
}

fn boundary_adjacency(input: &PolyData) -> BTreeMap<i64, BTreeSet<i64>> {
    let mut edge_count: BTreeMap<(i64, i64), usize> = BTreeMap::new();
    let num_points = input.points.len() as i64;

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            if a < 0 || b < 0 || a >= num_points || b >= num_points || a == b {
                continue;
            }
            *edge_count.entry(edge_key(a, b)).or_insert(0) += 1;
        }
    }

    let mut adjacency = BTreeMap::new();
    for ((a, b), count) in edge_count {
        if count == 1 {
            adjacency.entry(a).or_insert_with(BTreeSet::new).insert(b);
            adjacency.entry(b).or_insert_with(BTreeSet::new).insert(a);
        }
    }
    adjacency
}

fn edge_key(a: i64, b: i64) -> (i64, i64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_one_loop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let loops = extract_boundary_loops(&pd);
        assert_eq!(loops.lines.num_cells(), 1);
    }

    #[test]
    fn closed_surface_no_loops() {
        // Tetrahedron: 4 triangles, all edges shared
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(num_boundary_loops(&pd), 0);
    }

    #[test]
    fn quad_one_loop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(num_boundary_loops(&pd), 1);
    }

    #[test]
    fn disjoint_triangles_with_shared_point_extract_two_boundary_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 4]);

        let loops = extract_boundary_loops(&pd);
        assert_eq!(loops.lines.num_cells(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(num_boundary_loops(&pd), 0);
    }
}
