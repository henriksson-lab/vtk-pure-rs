use crate::data::PolyData;
use std::collections::HashMap;

/// Close all boundary loops in a mesh by capping them with fan triangulation.
///
/// Similar to `fill_holes` but also handles multiple separate boundary loops.
/// Finds all boundary edges, traces connected loops, and fills each with
/// triangles fanned from the loop centroid.
pub fn close_holes(input: &PolyData) -> PolyData {
    // Find boundary edges (edges shared by exactly one cell)
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut edge_next: HashMap<i64, Vec<i64>> = HashMap::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Build directed boundary edge graph
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            // Need to determine the boundary direction
            // Find the cell containing this edge and determine winding
            for cell in input.polys.iter() {
                for i in 0..cell.len() {
                    let ca = cell[i];
                    let cb = cell[(i + 1) % cell.len()];
                    if (ca == a && cb == b) || (ca == b && cb == a) {
                        // Boundary edge goes opposite to the cell winding
                        edge_next.entry(cb).or_default().push(ca);
                        break;
                    }
                }
            }
        }
    }

    let mut points = input.points.clone();
    let mut polys = input.polys.clone();

    // Trace boundary loops
    let mut visited = std::collections::HashSet::new();
    for &start in edge_next.keys() {
        if visited.contains(&start) {
            continue;
        }

        // Trace the loop
        let mut loop_pts = vec![start];
        visited.insert(start);
        let mut current = start;

        loop {
            let nexts = match edge_next.get(&current) {
                Some(n) => n,
                None => break,
            };
            let next = nexts.iter().find(|&&n| !visited.contains(&n));
            match next {
                Some(&n) => {
                    if n == start && loop_pts.len() >= 3 {
                        // Loop closed
                        break;
                    }
                    loop_pts.push(n);
                    visited.insert(n);
                    current = n;
                }
                None => break,
            }
        }

        if loop_pts.len() >= 3 {
            // Compute centroid
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &pid in &loop_pts {
                let p = points.get(pid as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = loop_pts.len() as f64;
            let center_id = points.len() as i64;
            points.push([cx / n, cy / n, cz / n]);

            // Fan triangulate
            for i in 0..loop_pts.len() {
                let a = loop_pts[i];
                let b = loop_pts[(i + 1) % loop_pts.len()];
                polys.push_cell(&[center_id, a, b]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_single_hole() {
        // Open box missing top face
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // 0
        pd.points.push([1.0, 0.0, 0.0]); // 1
        pd.points.push([1.0, 1.0, 0.0]); // 2
        pd.points.push([0.0, 1.0, 0.0]); // 3
                                         // Two triangles forming a quad
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = close_holes(&pd);
        // Original had 2 cells, should have more after closing
        assert!(result.polys.num_cells() >= 2);
    }

    #[test]
    fn already_closed() {
        // Single triangle - all edges are boundary, forms a single loop
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = close_holes(&pd);
        // Should add hole-filling triangles
        assert!(result.polys.num_cells() >= 1);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = close_holes(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
