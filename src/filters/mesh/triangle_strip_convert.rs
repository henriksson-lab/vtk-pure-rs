use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

/// Convert triangle mesh to greedy triangle strips.
///
/// Uses a greedy approach: starts from an edge, greedily extends the
/// strip by alternately connecting to the next adjacent triangle.
/// Converts polys to strips for more efficient rendering.
pub fn triangles_to_greedy_strips(input: &PolyData) -> PolyData {
    triangles_to_greedy_strips_with_maximum_length(input, 1000)
}

/// Convert triangle mesh to greedy triangle strips, limiting strip/line length.
pub fn triangles_to_greedy_strips_with_maximum_length(
    input: &PolyData,
    maximum_length: usize,
) -> PolyData {
    let maximum_length = maximum_length.clamp(4, 100000);
    let cells: Vec<Vec<i64>> = input
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| c.to_vec())
        .collect();
    let nc = cells.len();
    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.verts = input.verts.clone();
    pd.lines = strip_lines(&input.lines, maximum_length);
    pd.strips = input.strips.clone();
    *pd.point_data_mut() = input.point_data().clone();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..3 {
            let a = c[i];
            let b = c[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut used = vec![false; nc];
    let mut strips = CellArray::new();
    let mut non_tri_polys = CellArray::new();

    // Greedy strip building
    for start_fi in 0..nc {
        if used[start_fi] {
            continue;
        }
        let strip = best_strip_from(start_fi, &cells, &edge_faces, &mut used, maximum_length);

        if strip.len() >= 3 {
            strips.push_cell(&strip);
        }
    }

    for strip in strips.iter() {
        pd.strips.push_cell(strip);
    }

    // Pass through non-triangle polys
    for cell in input.polys.iter() {
        if cell.len() != 3 {
            non_tri_polys.push_cell(cell);
        }
    }

    pd.polys = non_tri_polys;
    pd
}

fn best_strip_from(
    start_fi: usize,
    cells: &[Vec<i64>],
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    used: &mut [bool],
    maximum_length: usize,
) -> Vec<i64> {
    let tri = &cells[start_fi];
    used[start_fi] = true;

    let mut neighbor = None;
    let mut strip = Vec::new();
    for i in 0..3 {
        let a = tri[i];
        let b = tri[(i + 1) % 3];
        if let Some(fi) = first_unvisited_edge_neighbor(edge_faces, used, a, b) {
            strip.push(tri[(i + 2) % 3]);
            strip.push(a);
            strip.push(b);
            neighbor = Some(fi);
            break;
        }
    }

    if strip.is_empty() {
        return tri.clone();
    }

    while let Some(fi) = neighbor {
        used[fi] = true;
        let last_a = strip[strip.len() - 2];
        let last_b = strip[strip.len() - 1];
        let Some(&next_point) = cells[fi].iter().find(|&&v| v != last_a && v != last_b) else {
            break;
        };

        strip.push(next_point);
        if strip.len() >= maximum_length + 2 {
            break;
        }

        neighbor = first_unvisited_edge_neighbor(edge_faces, used, next_point, last_b);
    }

    strip
}

fn first_unvisited_edge_neighbor(
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    used: &[bool],
    a: i64,
    b: i64,
) -> Option<usize> {
    let key = if a < b { (a, b) } else { (b, a) };
    edge_faces.get(&key)?.iter().copied().find(|&fi| !used[fi])
}

/// Count how many strips would be produced.
pub fn count_strips(input: &PolyData) -> usize {
    let result = triangles_to_greedy_strips(input);
    result.strips.num_cells()
}

fn strip_lines(lines: &CellArray, maximum_length: usize) -> CellArray {
    let mut out = CellArray::new();
    let mut segments = Vec::new();

    for cell in lines.iter() {
        if cell.len() > 2 {
            out.push_cell(cell);
        } else if cell.len() == 2 {
            segments.push([cell[0], cell[1]]);
        }
    }

    if segments.is_empty() {
        return out;
    }

    let mut point_cells: HashMap<i64, Vec<usize>> = HashMap::new();
    for (ci, segment) in segments.iter().enumerate() {
        point_cells.entry(segment[0]).or_default().push(ci);
        point_cells.entry(segment[1]).or_default().push(ci);
    }

    let mut visited = vec![false; segments.len()];
    for cell_id in 0..segments.len() {
        if visited[cell_id] {
            continue;
        }

        visited[cell_id] = true;
        let segment = segments[cell_id];
        let mut line = Vec::from(segment);
        let mut neighbor = None;

        for i in 0..2 {
            line[0] = segment[i];
            line[1] = segment[(i + 1) % 2];
            if let Some(candidates) = point_cells.get(&line[1]) {
                neighbor = candidates
                    .iter()
                    .copied()
                    .find(|&ci| ci != cell_id && !visited[ci]);
            }
            if neighbor.is_some() {
                break;
            }
        }

        if let Some(next_id) = neighbor {
            extend_line(
                &segments,
                &point_cells,
                &mut visited,
                &mut line,
                next_id,
                maximum_length,
            );
        }
        out.push_cell(&line);
    }

    out
}

fn extend_line(
    segments: &[[i64; 2]],
    point_cells: &HashMap<i64, Vec<usize>>,
    visited: &mut [bool],
    line: &mut Vec<i64>,
    mut next_id: usize,
    maximum_length: usize,
) {
    loop {
        visited[next_id] = true;
        let segment = segments[next_id];
        let endpoint = line[line.len() - 1];
        let other = if segment[0] == endpoint {
            segment[1]
        } else {
            segment[0]
        };
        line.push(other);

        let Some(candidates) = point_cells.get(&other) else {
            break;
        };
        let Some(candidate) = candidates
            .iter()
            .copied()
            .find(|&ci| ci != next_id && !visited[ci])
        else {
            break;
        };

        if line.len() >= maximum_length + 1 {
            break;
        }
        next_id = candidate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_from_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = triangles_to_greedy_strips(&pd);
        assert!(result.strips.num_cells() >= 1);
        assert!(result.strips.iter().any(|strip| strip.len() == 4));
    }

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.strips.num_cells(), 1);
    }

    #[test]
    fn preserves_non_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]); // quad

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.polys.num_cells(), 1); // quad preserved
    }

    #[test]
    fn strip_count() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(count_strips(&pd), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(count_strips(&pd), 0);
    }

    #[test]
    fn preserves_lines_and_existing_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
    }

    #[test]
    fn joins_line_segments() {
        let mut pd = PolyData::new();
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[1, 2]);

        let result = triangles_to_greedy_strips(&pd);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.lines.iter().next().unwrap(), &[0, 1, 2]);
    }

    #[test]
    fn limits_triangle_strip_length() {
        let mut pd = PolyData::new();
        for i in 0..8 {
            pd.points.push([i as f64, (i % 2) as f64, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[4, 5, 6]);
        pd.polys.push_cell(&[5, 6, 7]);

        let result = triangles_to_greedy_strips_with_maximum_length(&pd, 4);
        assert!(result.strips.iter().all(|strip| strip.len() <= 6));
    }
}
