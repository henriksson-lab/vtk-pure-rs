//! Mesh topology analysis: genus, Euler characteristic, orientability.

use crate::data::{AnyDataArray, DataArray, PolyData};

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
    let f = mesh.polys.num_cells();

    // Count unique edges
    let mut edges_set: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

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
    let num_components = count_components(mesh, v);

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
        num_boundary_loops: num_boundary_loops,
        num_components,
        is_closed: boundary_edges.is_empty(),
        is_orientable,
    }
}

fn count_loops(edges: &[(usize, usize)]) -> usize {
    if edges.is_empty() {
        return 0;
    }
    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
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

fn count_components(mesh: &PolyData, n: usize) -> usize {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    let mut visited = vec![false; n];
    let mut components = 0;
    for start in 0..n {
        if visited[start] || adj[start].is_empty() {
            continue;
        }
        let mut queue = vec![start];
        while let Some(v) = queue.pop() {
            if visited[v] {
                continue;
            }
            visited[v] = true;
            for &nb in &adj[v] {
                queue.push(nb);
            }
        }
        components += 1;
    }
    components.max(1)
}

fn check_orientability(cells: &[Vec<i64>]) -> bool {
    // Build directed edge map: for each edge, check that it appears in opposite directions
    let mut directed: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in cells {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *directed.entry((a, b)).or_insert(0) += 1;
        }
    }
    // For orientable: each internal edge (a,b) should appear once as (a,b) and once as (b,a)
    for (&(a, b), &count) in &directed {
        if count > 1 {
            return false;
        } // same direction appears twice
        let reverse = directed.get(&(b, a)).unwrap_or(&0);
        if *reverse > 1 {
            return false;
        }
    }
    true
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
