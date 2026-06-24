//! Quick topology information queries.

use crate::data::PolyData;

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

    let mut edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut boundary = 0;
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edges.insert((a.min(b), a.max(b)));
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    for &c in ec.values() {
        if c == 1 {
            boundary += 1;
        }
    }

    let n_arrays_pt = mesh.point_data().num_arrays();
    let n_arrays_cd = mesh.cell_data().num_arrays();

    let mut bb_min = [f64::MAX; 3];
    let mut bb_max = [f64::MIN; 3];
    for i in 0..n_pts {
        let p = mesh.points.get(i);
        for j in 0..3 {
            bb_min[j] = bb_min[j].min(p[j]);
            bb_max[j] = bb_max[j].max(p[j]);
        }
    }

    format!(
        "Points: {n_pts}, Polys: {n_polys} (tri={n_tri} quad={n_quad} other={n_other}), Lines: {n_lines}, Verts: {n_verts}\n\
         Edges: {}, Boundary: {boundary}, Closed: {}\n\
         Point arrays: {n_arrays_pt}, Cell arrays: {n_arrays_cd}\n\
         Bounds: [{:.3},{:.3}]×[{:.3},{:.3}]×[{:.3},{:.3}]",
        edges.len(), boundary==0,
        bb_min[0],bb_max[0],bb_min[1],bb_max[1],bb_min[2],bb_max[2])
}

/// Check if mesh is a triangle-only mesh.
pub fn is_triangle_mesh(mesh: &PolyData) -> bool {
    mesh.polys.num_cells() > 0 && mesh.polys.iter().all(|c| c.len() == 3)
}

/// Check if mesh is closed (no boundary edges).
pub fn is_closed_mesh(mesh: &PolyData) -> bool {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    ec.values().all(|&c| c == 2)
}

/// Check if mesh is manifold (each edge shared by exactly 1 or 2 faces).
pub fn is_manifold_mesh(mesh: &PolyData) -> bool {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    ec.values().all(|&c| c <= 2)
}

/// Euler characteristic: V - E + F.
pub fn euler_characteristic(mesh: &PolyData) -> i64 {
    let v = mesh.points.len() as i64;
    let f = mesh.polys.num_cells() as i64;
    let mut edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edges.insert((a.min(b), a.max(b)));
        }
    }
    v - edges.len() as i64 + f
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn info() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = mesh_info_string(&mesh);
        assert!(s.contains("Points: 3"));
        assert!(s.contains("tri=1"));
    }
    #[test]
    fn tri_only() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(is_triangle_mesh(&mesh));
    }
    #[test]
    fn closed_tet() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        assert!(is_closed_mesh(&mesh));
        assert_eq!(euler_characteristic(&mesh), 2);
    }
    #[test]
    fn open() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(!is_closed_mesh(&mesh));
    }
}
