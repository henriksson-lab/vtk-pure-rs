//! Convert triangle mesh to quad-dominant mesh by merging triangle pairs.
use crate::data::{CellArray, PolyData};

pub fn quad_remesh(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let tris: Vec<Option<[i64; 3]>> = cells
        .iter()
        .map(|c| (c.len() == 3).then_some([c[0], c[1], c[2]]))
        .collect();
    if tris.iter().filter(|tri| tri.is_some()).count() < 2 {
        return mesh.clone();
    }
    // Find shared edges between triangles
    let mut edge_tris: std::collections::HashMap<(i64, i64), Vec<usize>> =
        std::collections::HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        let Some([a, b, c]) = *tri else {
            continue;
        };
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
        let (Some(a), Some(b)) = (tris[t0], tris[t1]) else {
            continue;
        };
        // Find the shared edge vertices and the two opposite vertices
        let shared: Vec<i64> = a.iter().filter(|v| b.contains(v)).copied().collect();
        if shared.len() != 2 {
            continue;
        }
        let opp_a = a.iter().find(|v| !shared.contains(v)).copied();
        let opp_b = b.iter().find(|v| !shared.contains(v)).copied();
        if let (Some(oa), Some(ob)) = (opp_a, opp_b) {
            polys.push_cell(&[oa, shared[0], ob, shared[1]]);
            used[t0] = true;
            used[t1] = true;
        }
    }
    // Add remaining un-merged cells
    for (ti, c) in cells.iter().enumerate() {
        if !used[ti] {
            polys.push_cell(c);
        }
    }
    let mut result = mesh.clone();
    result.polys = polys;
    result.cell_data_mut().clear();
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

    #[test]
    fn preserves_non_triangle_cells() {
        let mut mesh = PolyData::new();
        for point in [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ] {
            mesh.points.push(point);
        }
        mesh.polys.push_cell(&[0, 1, 3]);
        mesh.polys.push_cell(&[0, 3, 2]);
        mesh.polys.push_cell(&[4, 5, 6, 7]);

        let r = quad_remesh(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
        assert!(r.polys.iter().any(|cell| cell == [4, 5, 6, 7]));
    }
}
