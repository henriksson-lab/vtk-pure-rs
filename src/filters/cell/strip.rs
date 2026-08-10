use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct EdgeEntry {
    hi: i64,
    tri_id: usize,
    edge_id: usize,
}

struct EdgeTable {
    buckets: Vec<Vec<EdgeEntry>>,
}

impl EdgeTable {
    fn new(num_points: usize) -> Self {
        Self {
            buckets: vec![Vec::new(); num_points],
        }
    }

    fn insert(
        &mut self,
        tri_verts: &[[i64; 3]],
        neighbors: &mut [[(usize, i64); 3]],
        a: i64,
        b: i64,
        tri_id: usize,
        edge_id: usize,
    ) {
        let Some((lo, hi)) = edge_ids(a, b, self.buckets.len()) else {
            return;
        };
        let bucket = &mut self.buckets[lo];
        if let Some(entry) = bucket.iter().find(|entry| entry.hi == hi) {
            neighbors[entry.tri_id][entry.edge_id] =
                (tri_id, unique_vertex(tri_verts[tri_id], a, b));
            neighbors[tri_id][edge_id] =
                (entry.tri_id, unique_vertex(tri_verts[entry.tri_id], a, b));
        } else {
            bucket.push(EdgeEntry {
                hi,
                tri_id,
                edge_id,
            });
        }
    }
}

fn edge_ids(a: i64, b: i64, num_points: usize) -> Option<(usize, i64)> {
    if a < 0 || b < 0 {
        return None;
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    let lo = lo as usize;
    if lo < num_points {
        Some((lo, hi))
    } else {
        None
    }
}

fn unique_vertex(tri: [i64; 3], a: i64, b: i64) -> i64 {
    tri.into_iter().find(|&v| v != a && v != b).unwrap_or(-1)
}

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
        output.strips = input.strips.clone();
        return output;
    }

    let mut edge_tris = EdgeTable::new(input.points.len());
    let mut neighbors = vec![[(usize::MAX, -1); 3]; nt];
    for (ti, tri) in tri_verts.iter().enumerate() {
        for (edge_id, edge) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])]
            .into_iter()
            .enumerate()
        {
            edge_tris.insert(&tri_verts, &mut neighbors, edge.0, edge.1, ti, edge_id);
        }
    }

    let mut visited = vec![false; nt];
    let mut strip_offsets = Vec::with_capacity(input.strips.num_cells() + nt + 1);
    let mut strip_connectivity =
        Vec::with_capacity(input.strips.connectivity_len() + nt.saturating_mul(3));
    strip_offsets.extend_from_slice(input.strips.offsets());
    strip_connectivity.extend_from_slice(input.strips.connectivity());

    let mut strip = Vec::with_capacity(maximum_length.saturating_add(2));
    let mut prefix = Vec::with_capacity(maximum_length.saturating_add(2));
    for start in 0..nt {
        if visited[start] {
            continue;
        }
        visited[start] = true;

        let tri = tri_verts[start];
        strip.clear();
        prefix.clear();
        strip.extend_from_slice(&tri);

        if let Some(([p0, p1, p2], next_ti)) =
            find_start_edge(&tri_verts, &neighbors, &visited, start, tri)
        {
            strip.clear();
            strip.extend_from_slice(&[p0, p1, p2]);
            visited[next_ti] = true;
            append_triangle(&mut strip, tri_verts[next_ti]);

            extend_strip_forward(
                &tri_verts,
                &neighbors,
                &mut visited,
                &mut strip,
                next_ti,
                maximum_length,
            );
            extend_strip_backward(
                &tri_verts,
                &neighbors,
                &mut visited,
                &mut prefix,
                start,
                [p0, p1],
                maximum_length.saturating_add(2).saturating_sub(strip.len()),
            );
        }

        strip_connectivity.extend(prefix.iter().rev().copied());
        strip_connectivity.extend(strip.iter().copied());
        strip_offsets.push(strip_connectivity.len() as i64);
    }

    output.strips = CellArray::from_raw(strip_offsets, strip_connectivity);
    output
}

fn find_start_edge(
    _tri_verts: &[[i64; 3]],
    neighbors: &[[(usize, i64); 3]],
    visited: &[bool],
    tri_id: usize,
    tri: [i64; 3],
) -> Option<([i64; 3], usize)> {
    for i in 0..3 {
        let (ti, _) = neighbors[tri_id][i];
        if ti != usize::MAX && !visited[ti] {
            let edge = (tri[i], tri[(i + 1) % 3]);
            return Some(([tri[(i + 2) % 3], edge.0, edge.1], ti));
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

fn extend_strip_forward(
    tri_verts: &[[i64; 3]],
    neighbors: &[[(usize, i64); 3]],
    visited: &mut [bool],
    strip: &mut Vec<i64>,
    mut last_tri: usize,
    maximum_length: usize,
) {
    while strip.len() < maximum_length + 2 {
        let n = strip.len();
        let Some(edge_id) = find_triangle_edge(tri_verts[last_tri], strip[n - 2], strip[n - 1])
        else {
            break;
        };
        let next = neighbors[last_tri][edge_id];
        if next.0 == usize::MAX || visited[next.0] {
            break;
        }

        visited[next.0] = true;
        strip.push(next.1);
        last_tri = next.0;
    }
}

fn extend_strip_backward(
    tri_verts: &[[i64; 3]],
    neighbors: &[[(usize, i64); 3]],
    visited: &mut [bool],
    prefix: &mut Vec<i64>,
    mut first_tri: usize,
    edge: [i64; 2],
    room: usize,
) {
    let mut edge = edge;
    while prefix.len() < room {
        let Some(edge_id) = find_triangle_edge(tri_verts[first_tri], edge[0], edge[1]) else {
            break;
        };
        let next = neighbors[first_tri][edge_id];
        if next.0 == usize::MAX || visited[next.0] {
            break;
        }

        visited[next.0] = true;
        prefix.push(next.1);
        first_tri = next.0;
        edge = [next.1, edge[0]];
    }
}

fn find_triangle_edge(tri: [i64; 3], a: i64, b: i64) -> Option<usize> {
    [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])]
        .into_iter()
        .position(|edge| (edge.0 == a && edge.1 == b) || (edge.0 == b && edge.1 == a))
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
