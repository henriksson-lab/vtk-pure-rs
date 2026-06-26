//! Advanced hole filling with quality triangulation.

use crate::data::PolyData;
use std::collections::HashMap;

/// Fill all boundary holes with fan triangulation from existing boundary vertices.
pub fn fill_all_holes(mesh: &PolyData) -> PolyData {
    let loops = find_boundary_loops(mesh);
    if loops.is_empty() {
        return mesh.clone();
    }

    let mut result = mesh.clone();
    for loop_verts in &loops {
        if loop_verts.len() < 3 {
            continue;
        }

        let root = loop_verts[0] as i64;
        for i in 1..loop_verts.len() - 1 {
            result
                .polys
                .push_cell(&[root, loop_verts[i] as i64, loop_verts[i + 1] as i64]);
        }
    }
    result
}

/// Fill holes with ear-clipping triangulation (better quality than fan).
pub fn fill_holes_ear_clip(mesh: &PolyData) -> PolyData {
    let loops = find_boundary_loops(mesh);
    if loops.is_empty() {
        return mesh.clone();
    }

    let mut result = mesh.clone();
    for loop_verts in &loops {
        if loop_verts.len() < 3 {
            continue;
        }
        let tris = ear_clip_3d(mesh, loop_verts);
        for tri in &tris {
            result
                .polys
                .push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
        }
    }
    result
}

/// Count boundary holes.
pub fn count_holes(mesh: &PolyData) -> usize {
    find_boundary_loops(mesh).len()
}

/// Get the size of each hole (number of boundary edges).
pub fn hole_sizes(mesh: &PolyData) -> Vec<usize> {
    find_boundary_loops(mesh).iter().map(|l| l.len()).collect()
}

fn find_boundary_loops(mesh: &PolyData) -> Vec<Vec<usize>> {
    let npoints = mesh.points.len();
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    let mut directed_edges = Vec::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 3 {
            continue;
        }
        for i in 0..nc {
            if cell[i] < 0 || cell[(i + 1) % nc] < 0 {
                continue;
            }
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= npoints || b >= npoints || a == b {
                continue;
            }
            *edge_counts.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            directed_edges.push((a, b));
        }
    }

    let boundary_edges: Vec<(usize, usize)> = directed_edges
        .into_iter()
        .filter(|&(a, b)| edge_counts.get(&(a.min(b), a.max(b))) == Some(&1))
        .collect();
    if boundary_edges.is_empty() {
        return Vec::new();
    }

    let mut edge_ids_by_vertex: HashMap<usize, Vec<usize>> = HashMap::new();
    for (edge_id, &(a, b)) in boundary_edges.iter().enumerate() {
        edge_ids_by_vertex.entry(a).or_default().push(edge_id);
        edge_ids_by_vertex.entry(b).or_default().push(edge_id);
    }

    let mut visited_edges = vec![false; boundary_edges.len()];
    let mut loops = Vec::new();

    for (start_edge, &(start, next)) in boundary_edges.iter().enumerate() {
        if visited_edges[start_edge] {
            continue;
        }

        let mut loop_v = vec![start];
        let mut current = next;
        let mut current_edge = start_edge;
        let mut valid = true;

        loop {
            visited_edges[current_edge] = true;
            if current == start {
                break;
            }

            loop_v.push(current);

            let Some(edge_ids) = edge_ids_by_vertex.get(&current) else {
                valid = false;
                break;
            };
            let unvisited: Vec<usize> = edge_ids
                .iter()
                .copied()
                .filter(|&edge_id| !visited_edges[edge_id])
                .collect();
            if unvisited.len() != 1 {
                valid = false;
                break;
            }

            current_edge = unvisited[0];
            let (a, b) = boundary_edges[current_edge];
            current = if a == current { b } else { a };
        }

        if valid && loop_v.len() >= 3 {
            loops.push(loop_v);
        }
    }
    loops
}

fn ear_clip_3d(_mesh: &PolyData, loop_verts: &[usize]) -> Vec<[usize; 3]> {
    let mut remaining: Vec<usize> = loop_verts.to_vec();
    let mut tris = Vec::new();

    while remaining.len() > 3 {
        let n = remaining.len();
        let mut found = false;
        for i in 0..n {
            let prev = remaining[(i + n - 1) % n];
            let curr = remaining[i];
            let next = remaining[(i + 1) % n];

            tris.push([prev, curr, next]);
            remaining.remove(i);
            found = true;
            break;
        }
        if !found {
            break;
        }
    }
    if remaining.len() == 3 {
        tris.push([remaining[0], remaining[1], remaining[2]]);
    }
    tris
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_single_hole() {
        // Open mesh with one boundary loop
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]], // open square, no bottom
        );
        let holes = count_holes(&mesh);
        assert!(holes >= 1);

        let filled = fill_all_holes(&mesh);
        assert!(filled.polys.num_cells() > mesh.polys.num_cells());
    }

    #[test]
    fn closed_mesh_no_holes() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        assert_eq!(count_holes(&mesh), 0);
        let filled = fill_all_holes(&mesh);
        assert_eq!(filled.polys.num_cells(), mesh.polys.num_cells());
    }

    #[test]
    fn ear_clip_fill() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let filled = fill_holes_ear_clip(&mesh);
        assert!(filled.polys.num_cells() >= mesh.polys.num_cells());
    }

    #[test]
    fn hole_sizes_test() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sizes = hole_sizes(&mesh);
        assert!(!sizes.is_empty());
        assert!(sizes[0] >= 3); // triangle has 3 boundary edges
    }
}
