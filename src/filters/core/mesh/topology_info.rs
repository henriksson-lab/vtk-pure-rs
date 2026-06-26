//! Quick topology information queries.

use crate::data::PolyData;
use std::collections::{HashMap, HashSet};

/// Quick mesh info as a formatted string.
pub fn mesh_info_string(mesh: &PolyData) -> String {
    let n_pts = mesh.points.len();
    let n_polys = mesh.polys.num_cells();
    let n_lines = mesh.lines.num_cells();
    let n_verts = mesh.verts.num_cells();

    let mut n_tri = 0;
    let mut n_quad = 0;
    let mut n_other = 0;
    for cell in mesh.polys.iter() {
        match cell.len() {
            3 => n_tri += 1,
            4 => n_quad += 1,
            _ => n_other += 1,
        }
    }

    let edge_counts = edge_counts(mesh);
    let (n_edges, boundary, closed) = match edge_counts {
        Some(edge_counts) => {
            let boundary = edge_counts.values().filter(|&&count| count == 1).count();
            (edge_counts.len(), boundary, edge_counts.values().all(|&count| count == 2))
        }
        None => (0, 0, false),
    };

    let n_arrays_pt = mesh.point_data().num_arrays();
    let n_arrays_cd = mesh.cell_data().num_arrays();

    let (bb_min, bb_max) = bounds(mesh);

    format!(
        "Points: {n_pts}, Polys: {n_polys} (tri={n_tri} quad={n_quad} other={n_other}), Lines: {n_lines}, Verts: {n_verts}\n\
         Edges: {n_edges}, Boundary: {boundary}, Closed: {closed}\n\
         Point arrays: {n_arrays_pt}, Cell arrays: {n_arrays_cd}\n\
         Bounds: [{:.3},{:.3}]×[{:.3},{:.3}]×[{:.3},{:.3}]",
        bb_min[0], bb_max[0], bb_min[1], bb_max[1], bb_min[2], bb_max[2]
    )
}

/// Check if mesh is a triangle-only mesh.
pub fn is_triangle_mesh(mesh: &PolyData) -> bool {
    mesh.polys.num_cells() > 0 && mesh.polys.iter().all(|c| c.len() == 3)
}

/// Check if mesh is closed (no boundary edges).
pub fn is_closed_mesh(mesh: &PolyData) -> bool {
    edge_counts(mesh)
        .map(|ec| ec.values().all(|&count| count == 2))
        .unwrap_or(false)
}

/// Check if mesh is manifold (each edge shared by exactly 1 or 2 faces).
pub fn is_manifold_mesh(mesh: &PolyData) -> bool {
    edge_counts(mesh)
        .map(|ec| ec.values().all(|&count| count <= 2))
        .unwrap_or(false)
}

/// Euler characteristic: V - E + F.
pub fn euler_characteristic(mesh: &PolyData) -> i64 {
    let v = mesh.points.len() as i64;
    let mut f = 0i64;
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    for cell in mesh.polys.iter() {
        if !is_valid_polygon(cell, mesh.points.len()) {
            continue;
        }
        f += 1;
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            edges.insert((a.min(b), a.max(b)));
        }
    }
    v - edges.len() as i64 + f
}

fn edge_counts(mesh: &PolyData) -> Option<HashMap<(usize, usize), usize>> {
    let mut ec: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in mesh.polys.iter() {
        if !is_valid_polygon(cell, mesh.points.len()) {
            return None;
        }
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    Some(ec)
}

fn is_valid_polygon(cell: &[i64], n_points: usize) -> bool {
    cell.len() >= 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < n_points)
}

fn bounds(mesh: &PolyData) -> ([f64; 3], [f64; 3]) {
    if mesh.points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }

    let mut bb_min = [f64::MAX; 3];
    let mut bb_max = [f64::MIN; 3];
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        for j in 0..3 {
            bb_min[j] = bb_min[j].min(p[j]);
            bb_max[j] = bb_max[j].max(p[j]);
        }
    }
    (bb_min, bb_max)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn info() {
        let mesh=PolyData::from_triangles(vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],vec![[0,1,2]]);
        let s=mesh_info_string(&mesh);
        assert!(s.contains("Points: 3"));
        assert!(s.contains("tri=1"));
    }
    #[test]
    fn tri_only() {
        let mesh=PolyData::from_triangles(vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],vec![[0,1,2]]);
        assert!(is_triangle_mesh(&mesh));
    }
    #[test]
    fn closed_tet() {
        let mesh=PolyData::from_triangles(
            vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,1.0]],
            vec![[0,1,2],[0,1,3],[1,2,3],[0,2,3]]);
        assert!(is_closed_mesh(&mesh));
        assert_eq!(euler_characteristic(&mesh),2);
    }
    #[test]
    fn open() {
        let mesh=PolyData::from_triangles(vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],vec![[0,1,2]]);
        assert!(!is_closed_mesh(&mesh));
    }
}
