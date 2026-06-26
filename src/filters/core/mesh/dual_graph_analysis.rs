//! Dual graph analysis: convert mesh to face adjacency graph and analyze.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Build the dual graph of a mesh: each face becomes a vertex, each
/// shared edge becomes an edge. Returns a PolyData with lines.
pub fn build_dual_graph(mesh: &PolyData) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();

    // Compute face centroids
    let mut centroids = Points::<f64>::new();
    for cell in &all_cells {
        let mut c = [0.0; 3];
        let mut k = 0.0;
        for &pid in cell {
            let Ok(pid) = usize::try_from(pid) else {
                continue;
            };
            if pid >= mesh.points.len() {
                continue;
            }
            let p = mesh.points.get(pid);
            for j in 0..3 {
                c[j] += p[j];
            }
            k += 1.0;
        }
        if k == 0.0 {
            centroids.push([0.0; 3]);
        } else {
            centroids.push([c[0] / k, c[1] / k, c[2] / k]);
        }
    }

    // Build edge→face adjacency
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let (Ok(a), Ok(b)) = (
                usize::try_from(cell[i]),
                usize::try_from(cell[(i + 1) % nc]),
            ) else {
                continue;
            };
            if a >= mesh.points.len() || b >= mesh.points.len() {
                continue;
            }
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    // Build dual edges
    let mut lines = CellArray::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for (_, faces) in &edge_faces {
        if faces.len() == 2 {
            let edge = (faces[0].min(faces[1]), faces[0].max(faces[1]));
            if seen.insert(edge) {
                lines.push_cell(&[edge.0 as i64, edge.1 as i64]);
            }
        }
    }

    // Compute face degree
    let mut degree = vec![0.0f64; n_cells];
    for (_, faces) in &edge_faces {
        if faces.len() == 2 {
            degree[faces[0]] += 1.0;
            degree[faces[1]] += 1.0;
        }
    }

    let mut result = PolyData::new();
    result.points = centroids;
    result.lines = lines;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceDegree",
            degree,
            1,
        )));
    result
}

/// Compute the graph diameter of the dual graph (longest shortest path).
pub fn dual_graph_diameter(mesh: &PolyData) -> usize {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n = all_cells.len();
    if n == 0 {
        return 0;
    }

    // Build adjacency
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let (Ok(a), Ok(b)) = (
                usize::try_from(cell[i]),
                usize::try_from(cell[(i + 1) % nc]),
            ) else {
                continue;
            };
            if a >= mesh.points.len() || b >= mesh.points.len() {
                continue;
            }
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (_, faces) in &edge_faces {
        if faces.len() == 2 {
            adj[faces[0]].push(faces[1]);
            adj[faces[1]].push(faces[0]);
        }
    }

    // BFS from every face to compute the exact diameter. Face order in a
    // PolyData is arbitrary, so fixed-index sampling can miss the endpoints.
    let mut max_dist = 0;
    for start in 0..n {
        let mut dist = vec![usize::MAX; n];
        dist[start] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if dist[nb] == usize::MAX {
                    dist[nb] = dist[v] + 1;
                    queue.push_back(nb);
                }
            }
        }
        let farthest = dist
            .iter()
            .filter(|&&d| d < usize::MAX)
            .cloned()
            .max()
            .unwrap_or(0);
        max_dist = max_dist.max(farthest);
    }
    max_dist
}

/// Color faces by their distance from a seed face in the dual graph.
pub fn dual_distance_from_face(mesh: &PolyData, seed_face: usize) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n = all_cells.len();

    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let (Ok(a), Ok(b)) = (
                usize::try_from(cell[i]),
                usize::try_from(cell[(i + 1) % nc]),
            ) else {
                continue;
            };
            if a >= mesh.points.len() || b >= mesh.points.len() {
                continue;
            }
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (_, faces) in &edge_faces {
        if faces.len() == 2 {
            adj[faces[0]].push(faces[1]);
            adj[faces[1]].push(faces[0]);
        }
    }

    let mut dist = vec![f64::MAX; n];
    if seed_face < n {
        dist[seed_face] = 0.0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(seed_face);
        while let Some(v) = queue.pop_front() {
            let d = dist[v] + 1.0;
            for &nb in &adj[v] {
                if d < dist[nb] {
                    dist[nb] = d;
                    queue.push_back(nb);
                }
            }
        }
    }
    for d in &mut dist {
        if *d == f64::MAX {
            *d = -1.0;
        }
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DualDistance",
            dist,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dual_of_grid() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..3 {
            for x in 0..3 {
                let bl = y * 4 + x;
                tris.push([bl, bl + 1, bl + 5]);
                tris.push([bl, bl + 5, bl + 4]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let dual = build_dual_graph(&mesh);
        assert_eq!(dual.points.len(), 18); // 3*3*2 triangles
        assert!(dual.lines.num_cells() > 0);
        assert!(dual.point_data().get_array("FaceDegree").is_some());
    }
    #[test]
    fn diameter() {
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
        let d = dual_graph_diameter(&mesh);
        assert!(d > 0);
    }

    #[test]
    fn diameter_is_exact_with_arbitrary_cell_order() {
        let path_order = [4usize, 1, 7, 0, 6, 2, 9, 3, 8, 5];
        let n_faces = path_order.len();
        let mut cells = vec![Vec::new(); n_faces];
        let mut point_count = 2 * (n_faces - 1);

        for i in 0..n_faces {
            let mut cell = Vec::new();
            if i > 0 {
                cell.push((2 * (i - 1) + 1) as i64);
                cell.push((2 * (i - 1)) as i64);
            } else {
                cell.push(point_count as i64);
                point_count += 1;
            }
            if i + 1 < n_faces {
                cell.push((2 * i) as i64);
                cell.push((2 * i + 1) as i64);
            } else {
                cell.push(point_count as i64);
                point_count += 1;
            }
            cells[path_order[i]] = cell;
        }

        let mut mesh = PolyData::new();
        for i in 0..point_count {
            mesh.points.push([i as f64, 0.0, 0.0]);
        }
        for cell in cells {
            mesh.polys.push_cell(&cell);
        }

        assert_eq!(dual_graph_diameter(&mesh), n_faces - 1);
    }

    #[test]
    fn dual_distance() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = dual_distance_from_face(&mesh, 0);
        let arr = result.cell_data().get_array("DualDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
