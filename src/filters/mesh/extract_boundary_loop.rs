use std::collections::{BTreeMap, HashMap};

use crate::data::{CellArray, Points, PolyData};

/// Extract closed boundary loops from a mesh.
///
/// Boundary edges are those used by exactly one polygon. The filter traces
/// connected boundary edges into closed loops and returns a PolyData with
/// line cells forming those loops, along with the number of loops found.
pub fn extract_boundary_loops(input: &PolyData) -> (PolyData, usize) {
    // Build adjacency for boundary edges only
    let mut boundary_edges: Vec<(usize, usize)> = polygon_edge_counts(input)
        .iter()
        .filter_map(|(&(a, b), &count)| (count == 1).then_some((a, b)))
        .collect();
    boundary_edges.sort();

    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for (edge_id, &(a, b)) in boundary_edges.iter().enumerate() {
        adj.entry(a).or_default().push(edge_id);
        adj.entry(b).or_default().push(edge_id);
    }
    for edges in adj.values_mut() {
        edges.sort_by_key(|&edge_id| {
            let (a, b) = boundary_edges[edge_id];
            (a.min(b), a.max(b))
        });
    }

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut visited_edges = vec![false; boundary_edges.len()];
    let mut pt_map: HashMap<usize, i64> = HashMap::new();
    let mut num_loops: usize = 0;

    for start_edge in 0..boundary_edges.len() {
        if visited_edges[start_edge] {
            continue;
        }

        let (start, next) = boundary_edges[start_edge];
        let mut loop_ids: Vec<i64> = Vec::new();
        let mut cur = start;
        let mut next = next;
        let mut edge_id = start_edge;
        let mut closed = false;

        loop {
            visited_edges[edge_id] = true;

            let mapped_cur: i64 = *pt_map.entry(cur).or_insert_with(|| {
                let idx: i64 = out_points.len() as i64;
                out_points.push(input.points.get(cur));
                idx
            });
            if loop_ids.is_empty() {
                loop_ids.push(mapped_cur);
            }

            let mapped_next: i64 = *pt_map.entry(next).or_insert_with(|| {
                let idx: i64 = out_points.len() as i64;
                out_points.push(input.points.get(next));
                idx
            });
            loop_ids.push(mapped_next);

            if next == start {
                closed = true;
                break;
            }

            let Some(next_edges) = adj.get(&next) else {
                break;
            };
            let Some(&next_edge_id) = next_edges
                .iter()
                .find(|&&candidate| !visited_edges[candidate])
            else {
                break;
            };

            let (a, b) = boundary_edges[next_edge_id];
            cur = next;
            next = if a == cur { b } else { a };
            edge_id = next_edge_id;
        }

        if !closed {
            continue;
        }

        if loop_ids.len() >= 4 {
            if let (Some(&first), Some(&last)) = (loop_ids.first(), loop_ids.last()) {
                if first != last {
                    continue;
                }
            }
            out_lines.push_cell(&loop_ids);
            num_loops += 1;
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    (pd, num_loops)
}

fn polygon_edge_counts(input: &PolyData) -> BTreeMap<(usize, usize), usize> {
    let mut edge_count = BTreeMap::new();
    let n_points = input.points.len();

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_counted_edge(
                &mut edge_count,
                n_points,
                cell[i],
                cell[(i + 1) % cell.len()],
            );
        }
    }

    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            insert_counted_edge(&mut edge_count, n_points, tri[0], tri[1]);
            insert_counted_edge(&mut edge_count, n_points, tri[1], tri[2]);
            insert_counted_edge(&mut edge_count, n_points, tri[2], tri[0]);
        }
    }

    edge_count
}

fn insert_counted_edge(
    edge_count: &mut BTreeMap<(usize, usize), usize>,
    n_points: usize,
    a: i64,
    b: i64,
) {
    let (Some(a), Some(b)) = (
        valid_point_index(a, n_points),
        valid_point_index(b, n_points),
    ) else {
        return;
    };
    if a != b {
        *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
    }
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
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

        let (result, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 1);
        assert_eq!(result.lines.num_cells(), 1);
        let line = result.lines.cell(0);
        assert_eq!(line.len(), 4);
        assert_eq!(line.first(), line.last());
        // The loop should contain 3 unique boundary vertices
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn closed_tetrahedron_no_loops() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[0, 2, 3]);

        let (_, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 0);
    }

    #[test]
    fn two_separate_triangles_two_loops() {
        let mut pd = PolyData::new();
        // First triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        // Second triangle (separate)
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([4.0, 0.0, 0.0]);
        pd.points.push([3.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let (_, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 2);
    }

    #[test]
    fn vertex_touching_triangles_are_two_loops() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 4]);

        let (result, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 2);
        assert_eq!(result.lines.num_cells(), 2);
    }

    #[test]
    fn invalid_polygon_ids_are_ignored() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 100]);

        let (_, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 1);
    }
}
