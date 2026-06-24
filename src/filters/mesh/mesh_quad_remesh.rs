//! Convert triangle mesh to quad-dominant mesh by merging triangle pairs.
use crate::data::{CellArray, Points, PolyData};

pub fn quad_remesh(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.len() < 2 {
        return mesh.clone();
    }
    // Find shared edges between triangles
    let mut edge_tris: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ti, &[a, b, c]) in tris.iter().enumerate() {
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
            edge_tris.entry(e).or_default().push(ti);
        }
    }
    let mut used = vec![false; tris.len()];
    let mut polys = CellArray::new();
    // Greedily merge triangle pairs sharing an edge
    for (_, tri_list) in &edge_tris {
        if tri_list.len() != 2 {
            continue;
        }
        let t0 = tri_list[0];
        let t1 = tri_list[1];
        if used[t0] || used[t1] {
            continue;
        }
        let a = tris[t0];
        let b = tris[t1];
        // Find the shared edge vertices and the two opposite vertices
        let shared: Vec<usize> = a.iter().filter(|v| b.contains(v)).copied().collect();
        if shared.len() != 2 {
            continue;
        }
        let opp_a = a.iter().find(|v| !shared.contains(v)).copied();
        let opp_b = b.iter().find(|v| !shared.contains(v)).copied();
        if let (Some(oa), Some(ob)) = (opp_a, opp_b) {
            polys.push_cell(&[oa as i64, shared[0] as i64, ob as i64, shared[1] as i64]);
            used[t0] = true;
            used[t1] = true;
        }
    }
    // Add remaining un-merged triangles
    for (ti, &[a, b, c]) in tris.iter().enumerate() {
        if !used[ti] {
            polys.push_cell(&[a as i64, b as i64, c as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quad() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = quad_remesh(&mesh);
        assert!(r.polys.num_cells() >= 1);
        // Should have merged into a quad
        let first = r.polys.iter().next().unwrap();
        assert_eq!(first.len(), 4);
    }
}
