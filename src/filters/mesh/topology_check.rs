use crate::data::PolyData;
use std::collections::HashMap;

/// Comprehensive mesh topology validation report.
#[derive(Debug, Clone)]
pub struct TopologyReport {
    pub num_vertices: usize,
    pub num_faces: usize,
    pub num_edges: usize,
    pub num_boundary_edges: usize,
    pub num_non_manifold_edges: usize,
    pub num_isolated_vertices: usize,
    pub euler_characteristic: i64,
    pub is_closed: bool,
    pub is_manifold: bool,
    pub is_oriented: bool,
}

/// Validate mesh topology and report issues.
pub fn topology_check(input: &PolyData) -> TopologyReport {
    let n_verts = input.points.len();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_faces = cells.len();

    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut directed_edges: HashMap<(i64, i64), usize> = HashMap::new();
    let mut vertex_used = vec![false; n_verts];

    for c in &cells {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            vertex_used[a as usize] = true;
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
            *directed_edges.entry((a, b)).or_insert(0) += 1;
        }
    }

    let n_edges = edge_count.len();
    let n_boundary = edge_count.values().filter(|&&c| c == 1).count();
    let n_non_manifold = edge_count.values().filter(|&&c| c > 2).count();
    let n_isolated = vertex_used.iter().filter(|&&u| !u).count();

    // Check orientation: each directed edge should appear at most once
    let is_oriented = directed_edges.values().all(|&c| c <= 1);

    TopologyReport {
        num_vertices: n_verts,
        num_faces: n_faces,
        num_edges: n_edges,
        num_boundary_edges: n_boundary,
        num_non_manifold_edges: n_non_manifold,
        num_isolated_vertices: n_isolated,
        euler_characteristic: n_verts as i64 - n_edges as i64 + n_faces as i64,
        is_closed: n_boundary == 0,
        is_manifold: n_non_manifold == 0,
        is_oriented,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tetrahedron() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let r = topology_check(&pd);
        assert_eq!(r.num_vertices, 4);
        assert_eq!(r.num_faces, 4);
        assert_eq!(r.euler_characteristic, 2);
        assert!(r.is_closed);
        assert!(r.is_manifold);
    }

    #[test]
    fn open_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let r = topology_check(&pd);
        assert!(!r.is_closed);
        assert_eq!(r.num_boundary_edges, 3);
    }

    #[test]
    fn non_manifold() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[0, 1, 4]);

        let r = topology_check(&pd);
        assert!(!r.is_manifold);
        assert!(r.num_non_manifold_edges > 0);
    }

    #[test]
    fn isolated_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([10.0, 10.0, 10.0]); // isolated
        pd.polys.push_cell(&[0, 1, 2]);

        let r = topology_check(&pd);
        assert_eq!(r.num_isolated_vertices, 1);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let r = topology_check(&pd);
        assert!(r.is_closed);
        assert!(r.is_manifold);
    }
}
