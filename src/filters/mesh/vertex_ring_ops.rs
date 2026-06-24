//! Vertex ring (neighborhood) operations.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract the 1-ring neighborhood of a vertex as a PolyData.
pub fn extract_one_ring(mesh: &PolyData, vertex: usize) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut selected = Vec::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        if cell.iter().any(|&pid| pid as usize == vertex) {
            selected.push(ci);
        }
    }
    extract_cells(mesh, &all_cells, &selected)
}

/// Extract the N-ring neighborhood of a vertex.
pub fn extract_n_ring(mesh: &PolyData, vertex: usize, n: usize) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();
    let np = mesh.points.len();
    let adj = build_adj(mesh, np);

    // Find vertices within N hops
    let mut ring_verts: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut frontier = vec![vertex];
    ring_verts.insert(vertex);
    for _ in 0..n {
        let mut next_frontier = Vec::new();
        for &v in &frontier {
            for &nb in &adj[v] {
                if ring_verts.insert(nb) {
                    next_frontier.push(nb);
                }
            }
        }
        frontier = next_frontier;
    }

    // Select cells that have all vertices in the ring
    let selected: Vec<usize> = (0..n_cells)
        .filter(|&ci| {
            all_cells[ci]
                .iter()
                .all(|&pid| ring_verts.contains(&(pid as usize)))
        })
        .collect();
    extract_cells(mesh, &all_cells, &selected)
}

/// Compute one-ring valence (number of edges at each vertex).
pub fn vertex_valence(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let data: Vec<f64> = adj.iter().map(|a| a.len() as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Valence", data, 1)));
    result
}

/// Find irregular vertices (valence != 6 for interior, != 4 for boundary).
pub fn find_irregular_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let boundary = find_boundary_verts(mesh, n);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let expected = if boundary[i] { 4 } else { 6 };
            if adj[i].len() != expected {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Irregular", data, 1)));
    result
}

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for c in m.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let x = c[i] as usize;
            let y = c[(i + 1) % nc] as usize;
            if x < n && y < n {
                a[x].insert(y);
                a[y].insert(x);
            }
        }
    }
    a.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn find_boundary_verts(mesh: &PolyData, n: usize) -> Vec<bool> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for c in mesh.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let a = c[i] as usize;
            let b = c[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut bnd = vec![false; n];
    for (&(a, b), &c) in &ec {
        if c == 1 {
            bnd[a] = true;
            bnd[b] = true;
        }
    }
    bnd
}

fn extract_cells(mesh: &PolyData, all_cells: &[Vec<i64>], selected: &[usize]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &ci in selected {
        let cell = &all_cells[ci];
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pm.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_ring() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let ring = extract_one_ring(&mesh, 12); // center vertex
        assert!(ring.polys.num_cells() > 0);
    }
    #[test]
    fn n_ring() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let r1 = extract_n_ring(&mesh, 12, 1);
        let r2 = extract_n_ring(&mesh, 12, 2);
        assert!(r2.polys.num_cells() > r1.polys.num_cells());
    }
    #[test]
    fn valence() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = vertex_valence(&mesh);
        assert!(result.point_data().get_array("Valence").is_some());
    }
    #[test]
    fn irregular() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = find_irregular_vertices(&mesh);
        assert!(result.point_data().get_array("Irregular").is_some());
    }
}
