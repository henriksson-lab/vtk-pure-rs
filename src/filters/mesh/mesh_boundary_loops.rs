//! Extract boundary loops (ordered closed polylines) from a mesh.
use crate::data::{CellArray, PolyData};

pub fn boundary_loops(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    // Find boundary edges (shared by exactly one face)
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        add_polygon_edges(cell, n, &mut edge_count);
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(strip, n, &mut edge_count);
    }
    let mut boundary_adj: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }
    let mut lines = CellArray::new();
    let mut visited_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    for &start in boundary_adj.keys() {
        while let Some(&first_next) = boundary_adj.get(&start).and_then(|neighbors| {
            neighbors
                .iter()
                .find(|&&nb| !visited_edges.contains(&edge_key(start, nb)))
        }) {
            // Trace one boundary chain/loop as a single polyline cell.
            let mut loop_verts = vec![start];
            let mut current = start;
            let mut next = first_next;

            loop {
                visited_edges.insert(edge_key(current, next));
                loop_verts.push(next);

                let previous = current;
                current = next;
                if current == start {
                    break;
                }

                let Some(neighbors) = boundary_adj.get(&current) else {
                    break;
                };
                let candidate = neighbors
                    .iter()
                    .copied()
                    .filter(|&nb| nb != previous)
                    .find(|&nb| !visited_edges.contains(&edge_key(current, nb)))
                    .or_else(|| {
                        neighbors
                            .iter()
                            .copied()
                            .find(|&nb| !visited_edges.contains(&edge_key(current, nb)))
                    });

                match candidate {
                    Some(nb) => next = nb,
                    None => break,
                }
            }

            if loop_verts.len() >= 2 {
                let line: Vec<i64> = loop_verts.into_iter().map(|v| v as i64).collect();
                lines.push_cell(&line);
            }
        }
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.lines = lines;
    result
}

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

fn add_polygon_edges(
    cell: &[i64],
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    let nc = cell.len();
    if nc < 2 || !valid_cell(cell, npoints) {
        return;
    }
    for i in 0..nc {
        add_edge(cell[i] as usize, cell[(i + 1) % nc] as usize, edge_count);
    }
}

fn add_triangle_strip_edges(
    strip: &[i64],
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    if strip.len() < 3 || !valid_cell(strip, npoints) {
        return;
    }
    for i in 0..strip.len() - 2 {
        let tri = if i % 2 == 0 {
            [strip[i], strip[i + 1], strip[i + 2]]
        } else {
            [strip[i + 1], strip[i], strip[i + 2]]
        };
        add_edge(tri[0] as usize, tri[1] as usize, edge_count);
        add_edge(tri[1] as usize, tri[2] as usize, edge_count);
        add_edge(tri[2] as usize, tri[0] as usize, edge_count);
    }
}

fn add_edge(a: usize, b: usize, edge_count: &mut std::collections::HashMap<(usize, usize), u32>) {
    if a == b {
        return;
    }
    *edge_count.entry(edge_key(a, b)).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boundary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boundary_loops(&mesh);
        assert_eq!(r.lines.num_cells(), 1); // triangle has 3 boundary edges forming 1 loop
    }

    #[test]
    fn triangle_strips_are_decomposed() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let r = boundary_loops(&mesh);

        assert_eq!(r.lines.num_cells(), 1);
        assert_eq!(r.lines.cell(0).len(), 5);
    }
}
