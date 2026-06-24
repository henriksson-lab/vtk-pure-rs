use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extract all boundary loops as separate polyline cells.
///
/// Each boundary loop is a closed polyline of edges that belong to
/// exactly one triangle. Returns a PolyData with line cells.
pub fn extract_boundary_loops(input: &PolyData) -> PolyData {
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut boundary_next: HashMap<i64, Vec<i64>> = HashMap::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Build directed boundary edges
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary_next.entry(a).or_default().push(b);
            boundary_next.entry(b).or_default().push(a);
        }
    }

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut visited = std::collections::HashSet::new();
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
        if visited.contains(&start) {
            continue;
        }

        let mut loop_ids = Vec::new();
        let mut cur = start;

        loop {
            if visited.contains(&cur) && !loop_ids.is_empty() {
                break;
            }
            visited.insert(cur);
            let mapped = map_pt(cur, input, &mut out_points, &mut pt_map);
            loop_ids.push(mapped);

            let nexts = match boundary_next.get(&cur) {
                Some(v) => v,
                None => break,
            };
            match nexts.iter().find(|&&n| !visited.contains(&n)) {
                Some(&n) => cur = n,
                None => {
                    // Close loop if possible
                    if nexts.contains(&start) && loop_ids.len() >= 3 {
                        loop_ids.push(loop_ids[0]); // close
                    }
                    break;
                }
            }
        }

        if loop_ids.len() >= 3 {
            out_lines.push_cell(&loop_ids);
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
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(num_boundary_loops(&pd), 0);
    }
}
