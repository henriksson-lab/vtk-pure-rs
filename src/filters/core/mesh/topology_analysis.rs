//! Mesh topology analysis: genus, Euler characteristic, orientability.

use crate::data::PolyData;
use std::collections::{HashMap, HashSet};

/// Topology analysis result.
#[derive(Debug, Clone)]
pub struct TopologyAnalysis {
    pub vertices: usize,
    pub edges: usize,
    pub faces: usize,
    pub euler_characteristic: i64,
    pub genus: i64,
    pub num_boundary_loops: usize,
    pub num_components: usize,
    pub is_closed: bool,
    pub is_orientable: bool,
}

impl std::fmt::Display for TopologyAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "V={} E={} F={} χ={} g={} loops={} components={} closed={} orientable={}",
            self.vertices,
            self.edges,
            self.faces,
            self.euler_characteristic,
            self.genus,
            self.num_boundary_loops,
            self.num_components,
            self.is_closed,
            self.is_orientable
        )
    }
}

/// Compute comprehensive topology analysis.
pub fn analyze_topology(mesh: &PolyData) -> TopologyAnalysis {
    let v = mesh.points.len();
    let all_cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|cell| is_valid_polygon(cell, v))
        .map(|c| c.to_vec())
        .collect();
    let f = all_cells.len();

    // Count unique edges
    let mut edges_set: HashSet<(usize, usize)> = HashSet::new();
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for cell in &all_cells {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let edge = (a.min(b), a.max(b));
            edges_set.insert(edge);
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    }

    let e = edges_set.len();
    let chi = v as i64 - e as i64 + f as i64;

    // Boundary loops
    let boundary_edges: Vec<(usize, usize)> = edge_count
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    let num_boundary_loops = count_loops(&boundary_edges);

    // Connected components
    let num_components = count_components(&all_cells);

    // Genus: χ = 2(c - g) - b for orientable surfaces
    // c = components, b = boundary loops, g = genus
    let genus = (2 * num_components as i64 - chi - num_boundary_loops as i64) / 2;

    // Orientability check (simplified: check for consistent edge orientation)
    let is_orientable = check_orientability(&all_cells);

    TopologyAnalysis {
        vertices: v,
        edges: e,
        faces: f,
        euler_characteristic: chi,
        genus: genus.max(0),
        num_boundary_loops,
        num_components,
        is_closed: boundary_edges.is_empty(),
        is_orientable,
    }
}

fn count_loops(edges: &[(usize, usize)]) -> usize {
    if edges.is_empty() {
        return 0;
    }
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    let mut visited: HashSet<usize> = HashSet::new();
    let mut loops = 0;
    for &(start, _) in edges {
        if visited.contains(&start) {
            continue;
        }
        let mut queue = vec![start];
        while let Some(v) = queue.pop() {
            if !visited.insert(v) {
                continue;
            }
            if let Some(neighbors) = adj.get(&v) {
                for &n in neighbors {
                    queue.push(n);
                }
            }
        }
        loops += 1;
    }
    loops
}

fn count_components(cells: &[Vec<i64>]) -> usize {
    if cells.is_empty() {
        return 0;
    }

    let mut point_cells: HashMap<i64, Vec<usize>> = HashMap::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        for &point_id in cell {
            point_cells.entry(point_id).or_default().push(cell_id);
        }
    }

    let mut visited = vec![false; cells.len()];
    let mut components = 0;
    for start in 0..cells.len() {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut stack = vec![start];
        while let Some(cell_id) = stack.pop() {
            if visited[cell_id] {
                continue;
            }
            visited[cell_id] = true;
            for &point_id in &cells[cell_id] {
                if let Some(neighbors) = point_cells.get(&point_id) {
                    for &neighbor in neighbors {
                        if !visited[neighbor] {
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }
    }
    components
}

fn check_orientability(cells: &[Vec<i64>]) -> bool {
    let mut edge_faces: HashMap<(usize, usize), Vec<(usize, bool)>> = HashMap::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let edge = (a.min(b), a.max(b));
            edge_faces.entry(edge).or_default().push((cell_id, a < b));
        }
    }

    let mut adjacency = vec![Vec::<(usize, bool)>::new(); cells.len()];
    for faces in edge_faces.values() {
        if faces.len() > 2 {
            return false;
        }
        if let [(face_a, dir_a), (face_b, dir_b)] = faces.as_slice() {
            // Adjacent faces must traverse their shared edge in opposite
            // directions after any face flips. Equal original directions
            // therefore require opposite flip states; opposite directions
            // require equal flip states.
            let same_flip = dir_a != dir_b;
            adjacency[*face_a].push((*face_b, same_flip));
            adjacency[*face_b].push((*face_a, same_flip));
        }
    }

    let mut orientation = vec![None; cells.len()];
    for start in 0..cells.len() {
        if orientation[start].is_some() {
            continue;
        }
        orientation[start] = Some(false);
        let mut stack = vec![start];
        while let Some(face_id) = stack.pop() {
            let face_orientation = orientation[face_id].unwrap();
            for &(neighbor, same_flip) in &adjacency[face_id] {
                let expected = if same_flip {
                    face_orientation
                } else {
                    !face_orientation
                };
                match orientation[neighbor] {
                    Some(actual) if actual != expected => return false,
                    Some(_) => {}
                    None => {
                        orientation[neighbor] = Some(expected);
                        stack.push(neighbor);
                    }
                }
            }
        }
    }
    true
}

fn is_valid_polygon(cell: &[i64], n_points: usize) -> bool {
    cell.len() >= 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tetrahedron() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let topo = analyze_topology(&mesh);
        assert_eq!(topo.vertices, 4);
        assert_eq!(topo.faces, 4);
        assert_eq!(topo.euler_characteristic, 2);
        assert!(topo.is_closed);
        assert!(topo.is_orientable);
    }

    #[test]
    fn open_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let topo = analyze_topology(&mesh);
        assert!(!topo.is_closed);
        assert_eq!(topo.num_boundary_loops, 1);
    }

    #[test]
    fn display() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let topo = analyze_topology(&mesh);
        let s = format!("{topo}");
        assert!(s.contains("V=3"));
    }

    #[test]
    fn two_components() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let topo = analyze_topology(&mesh);
        assert_eq!(topo.num_components, 2);
    }
}
