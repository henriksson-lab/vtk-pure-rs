use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

/// Convert triangle polygons to triangle strips.
///
/// Existing strips are passed through, non-triangle polygons stay in `polys`,
/// and triangle polygons are greedily assembled into new strips.
pub fn to_triangle_strips(input: &PolyData) -> PolyData {
    to_triangle_strips_with_maximum_length(input, 1000)
}

/// Convert triangle polygons to triangle strips, limiting strip/line length.
///
/// `maximum_length` matches `vtkStripper::MaximumLength`: at most this many
/// triangles per strip, and at most this many segments per poly-line.
pub fn to_triangle_strips_with_maximum_length(input: &PolyData, maximum_length: usize) -> PolyData {
    let maximum_length = maximum_length.clamp(4, 100000);
    let mut output = PolyData::new();
    output.points = input.points.clone();
    output.verts = input.verts.clone();
    output.lines = strip_lines(&input.lines, maximum_length);
    output.strips = input.strips.clone();
    *output.point_data_mut() = input.point_data().clone();

    let mut tri_verts = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            tri_verts.push([cell[0], cell[1], cell[2]]);
        } else {
            output.polys.push_cell(cell);
        }
    }

    let nt = tri_verts.len();
    if nt == 0 {
        return output;
    }

    let mut edge_tris: HashMap<(i64, i64), [usize; 2]> = HashMap::with_capacity(nt * 3);
    for (ti, tri) in tri_verts.iter().enumerate() {
        for edge in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = edge_key(edge.0, edge.1);
            let entry = edge_tris.entry(key).or_insert([usize::MAX, usize::MAX]);
            if entry[0] == usize::MAX {
                entry[0] = ti;
            } else if entry[1] == usize::MAX {
                entry[1] = ti;
            }
        }
    }

    let mut visited = vec![false; nt];
    for start in 0..nt {
        if visited[start] {
            continue;
        }
        visited[start] = true;

        let tri = tri_verts[start];
        let mut strip = Vec::from(tri);

        if let Some(([p0, p1, p2], next_ti)) =
            find_start_edge(&tri_verts, &edge_tris, &visited, tri)
        {
            strip.clear();
            strip.extend_from_slice(&[p0, p1, p2]);
            visited[next_ti] = true;
            append_triangle(&mut strip, tri_verts[next_ti]);

            loop {
                let n = strip.len();
                let Some(next) = find_unvisited_neighbor(
                    &tri_verts,
                    &edge_tris,
                    &visited,
                    strip[n - 2],
                    strip[n - 1],
                ) else {
                    break;
                };

                visited[next.0] = true;
                strip.push(next.1);
                if strip.len() >= maximum_length + 2 {
                    break;
                }
            }
        }

        output.strips.push_cell(&strip);
    }

    output
}

fn find_start_edge(
    _tri_verts: &[[i64; 3]],
    edge_tris: &HashMap<(i64, i64), [usize; 2]>,
    visited: &[bool],
    tri: [i64; 3],
) -> Option<([i64; 3], usize)> {
    for i in 0..3 {
        let edge = (tri[i], tri[(i + 1) % 3]);
        if let Some(pair) = edge_tris.get(&edge_key(edge.0, edge.1)) {
            for &ti in pair {
                if ti != usize::MAX && !visited[ti] {
                    return Some(([tri[(i + 2) % 3], edge.0, edge.1], ti));
                }
            }
        }
    }
    None
}

fn append_triangle(strip: &mut Vec<i64>, tri: [i64; 3]) {
    let n = strip.len();
    if let Some(v) = tri
        .into_iter()
        .find(|&v| v != strip[n - 2] && v != strip[n - 1])
    {
        strip.push(v);
    }
}

fn find_unvisited_neighbor(
    tri_verts: &[[i64; 3]],
    edge_tris: &HashMap<(i64, i64), [usize; 2]>,
    visited: &[bool],
    a: i64,
    b: i64,
) -> Option<(usize, i64)> {
    let pair = edge_tris.get(&edge_key(a, b))?;
    for &ti in pair {
        if ti == usize::MAX || visited[ti] {
            continue;
        }
        let tri = tri_verts[ti];
        for v in tri {
            if v != a && v != b {
                return Some((ti, v));
            }
        }
    }
    None
}

fn edge_key(a: i64, b: i64) -> (i64, i64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
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
        let mut line = Vec::from(segments[cell_id]);
        extend_line(
            &segments,
            &point_cells,
            &mut visited,
            &mut line,
            false,
            maximum_length,
        );
        extend_line(
            &segments,
            &point_cells,
            &mut visited,
            &mut line,
            true,
            maximum_length,
        );
        out.push_cell(&line);
    }

    out
}

fn extend_line(
    segments: &[[i64; 2]],
    point_cells: &HashMap<i64, Vec<usize>>,
    visited: &mut [bool],
    line: &mut Vec<i64>,
    at_front: bool,
    maximum_length: usize,
) {
    loop {
        if line.len() >= maximum_length + 1 {
            break;
        }
        let endpoint = if at_front {
            line[0]
        } else {
            line[line.len() - 1]
        };
        let Some(candidates) = point_cells.get(&endpoint) else {
            break;
        };
        let Some(&next_id) = candidates.iter().find(|&&ci| !visited[ci]) else {
            break;
        };
        visited[next_id] = true;
        let segment = segments[next_id];
        let other = if segment[0] == endpoint {
            segment[1]
        } else {
            segment[0]
        };
        if at_front {
            line.insert(0, other);
        } else {
            line.push(other);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_to_strip() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = to_triangle_strips(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.strips.cell(0).len(), 3);
    }

    #[test]
    fn two_adjacent_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = to_triangle_strips(&pd);
        let total_strip_verts: usize = result.strips.iter().map(|s| s.len()).sum();
        assert!(total_strip_verts <= 5);
    }

    #[test]
    fn preserves_points() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = to_triangle_strips(&pd);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn passes_existing_strips_and_non_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = to_triangle_strips(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
    }
}
