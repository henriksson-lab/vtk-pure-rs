//! Mesh topology queries (genus, Euler, manifold checks).

use crate::data::PolyData;

/// Topology information for a mesh.
pub struct TopologyInfo {
    pub vertices: usize,
    pub edges: usize,
    pub faces: usize,
    pub euler_characteristic: isize,
    pub genus: isize,
    pub boundary_loops: usize,
    pub is_manifold: bool,
    pub is_closed: bool,
}

/// Compute topology info for a mesh.
pub fn topology_info(mesh: &PolyData) -> TopologyInfo {
    let v = mesh.points.len();
    let f = mesh.polys.num_cells();
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let e = edge_count.len();
    let boundary = edge_count.values().filter(|&&c| c == 1).count();
    let non_manifold = edge_count.values().filter(|&&c| c > 2).count();

    // Count boundary loops
    let boundary_loops = count_boundary_loops(&edge_count, mesh);

    let euler = v as isize - e as isize + f as isize;
    // genus = (2 - euler - boundary_loops) / 2 for orientable surfaces
    let genus = (2 - euler - boundary_loops as isize) / 2;

    TopologyInfo {
        vertices: v,
        edges: e,
        faces: f,
        euler_characteristic: euler,
        genus: genus.max(0),
        boundary_loops,
        is_manifold: non_manifold == 0,
        is_closed: boundary == 0,
    }
}

fn count_boundary_loops(
    edge_count: &std::collections::HashMap<(usize, usize), usize>,
    _mesh: &PolyData,
) -> usize {
    let boundary_edges: Vec<(usize, usize)> = edge_count
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    if boundary_edges.is_empty() {
        return 0;
    }

    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &boundary_edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut loops = 0;
    for &start in adj.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut cur = start;
        loop {
            visited.insert(cur);
            let next = adj
                .get(&cur)
                .and_then(|nbs| nbs.iter().find(|&&n| !visited.contains(&n)));
            match next {
                Some(&n) => cur = n,
                None => break,
            }
        }
        loops += 1;
    }
    loops
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single_tri() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let info = topology_info(&mesh);
        assert_eq!(info.vertices, 3);
        assert_eq!(info.edges, 3);
        assert_eq!(info.faces, 1);
        assert!(!info.is_closed);
        assert!(info.is_manifold);
    }
    #[test]
    fn test_closed_tetra() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let info = topology_info(&mesh);
        assert!(info.is_closed);
        assert_eq!(info.euler_characteristic, 2); // sphere-like
    }
}
