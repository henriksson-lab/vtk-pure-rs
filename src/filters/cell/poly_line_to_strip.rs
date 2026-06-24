use crate::data::{CellArray, PolyData};

/// Convert multiple separate line cells into connected polylines.
///
/// Joins line segments that share endpoints into longer polylines.
/// Useful after operations that produce many 2-point line segments
/// (like intersection or contouring).
pub fn poly_line_to_strip(input: &PolyData) -> PolyData {
    use std::collections::HashMap;

    // Build adjacency from line endpoints
    let mut adj: HashMap<i64, Vec<(usize, i64)>> = HashMap::new(); // point -> [(cell_idx, other_point)]
    let segments: Vec<(i64, i64)> = input
        .lines
        .iter()
        .filter_map(|cell| {
            if cell.len() == 2 {
                Some((cell[0], cell[1]))
            } else {
                None
            }
        })
        .collect();

    for (ci, &(a, b)) in segments.iter().enumerate() {
        adj.entry(a).or_default().push((ci, b));
        adj.entry(b).or_default().push((ci, a));
    }

    let mut used = vec![false; segments.len()];
    let mut out_lines = CellArray::new();

    // Also pass through non-2-point lines as-is
    for cell in input.lines.iter() {
        if cell.len() != 2 {
            out_lines.push_cell(cell);
        }
    }

    // Trace connected chains
    for start_ci in 0..segments.len() {
        if used[start_ci] {
            continue;
        }

        // Find one end of the chain (a point with degree 1 or start)
        let (mut cur, mut prev) = (segments[start_ci].0, -1i64);

        // Walk backward to find chain start
        loop {
            let neighbors = adj.get(&cur).map(|v| v.as_slice()).unwrap_or(&[]);
            let next = neighbors
                .iter()
                .find(|&&(ci, other)| !used[ci] && other != prev);
            match next {
                Some(&(_, other)) if other != segments[start_ci].0 => {
                    prev = cur;
                    cur = other;
                }
                _ => break,
            }
        }

        // Now trace forward from cur
        let mut chain = vec![cur];
        prev = -1;
        loop {
            let neighbors = adj.get(&cur).map(|v| v.as_slice()).unwrap_or(&[]);
            let next = neighbors
                .iter()
                .find(|&&(ci, other)| !used[ci] && other != prev);
            match next {
                Some(&(ci, other)) => {
                    used[ci] = true;
                    chain.push(other);
                    prev = cur;
                    cur = other;
                }
                None => break,
            }
        }

        if chain.len() >= 2 {
            out_lines.push_cell(&chain);
        }
    }

    let mut pd = input.clone();
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_segments() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[1, 2]);
        pd.lines.push_cell(&[2, 3]);

        let result = poly_line_to_strip(&pd);
        assert_eq!(result.lines.num_cells(), 1); // joined into one polyline
        let cell: Vec<i64> = result.lines.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4); // 4 points in the chain
    }

    #[test]
    fn disconnected_segments() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[2, 3]);

        let result = poly_line_to_strip(&pd);
        assert_eq!(result.lines.num_cells(), 2); // two separate chains
    }

    #[test]
    fn already_polyline() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]); // already a polyline (3 pts)

        let result = poly_line_to_strip(&pd);
        assert_eq!(result.lines.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = poly_line_to_strip(&pd);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
