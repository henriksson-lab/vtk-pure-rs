//! Compute topological genus of a mesh using Euler characteristic.
use crate::data::PolyData;

pub fn genus(mesh: &PolyData) -> i64 {
    let used_points = used_polygon_points(mesh);
    let v = used_points.iter().filter(|&&used| used).count() as i64;
    let f = face_count(mesh) as i64;
    let (edges, edge_count) = edge_data(mesh);
    let e = edges.len() as i64;
    let chi = v - e + f;
    let boundary_edges: Vec<(i64, i64)> = edge_count
        .iter()
        .filter(|(_, &count)| count == 1)
        .map(|(&edge, _)| edge)
        .collect();
    let num_boundary_loops = count_boundary_loops(&boundary_edges) as i64;
    let num_components = count_components(mesh) as i64;

    // Euler characteristic: V - E + F = 2c - 2g - b for orientable surfaces.
    ((2 * num_components - chi - num_boundary_loops) / 2).max(0)
}

pub fn euler_characteristic(mesh: &PolyData) -> i64 {
    let used_points = used_polygon_points(mesh);
    let v = used_points.iter().filter(|&&used| used).count() as i64;
    let f = face_count(mesh) as i64;
    let (edges, _) = edge_data(mesh);
    v - edges.len() as i64 + f
}

fn edge_data(
    mesh: &PolyData,
) -> (
    std::collections::HashSet<(i64, i64)>,
    std::collections::HashMap<(i64, i64), usize>,
) {
    let mut edges = std::collections::HashSet::new();
    let mut edge_count = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if !valid_polygon(cell, mesh.points.len()) {
            continue;
        }
        for i in 0..nc {
            add_edge(
                cell[i],
                cell[(i + 1) % nc],
                mesh.points.len(),
                &mut edges,
                &mut edge_count,
            );
        }
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(strip, mesh.points.len(), &mut edges, &mut edge_count);
    }
    (edges, edge_count)
}

fn face_count(mesh: &PolyData) -> usize {
    mesh.polys
        .iter()
        .filter(|cell| valid_polygon(cell, mesh.points.len()))
        .count()
        + mesh
            .strips
            .iter()
            .map(|strip| {
                strip
                    .windows(3)
                    .filter(|tri| valid_triangle(tri, mesh.points.len()))
                    .count()
            })
            .sum::<usize>()
}

fn count_components(mesh: &PolyData) -> usize {
    let n = mesh.points.len();
    if n == 0 {
        return 0;
    }
    let used_points = used_polygon_points(mesh);
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if !valid_polygon(cell, n) {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            if a != b {
                adj[a].push(b);
                adj[b].push(a);
            }
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            if !valid_triangle(tri, n) {
                continue;
            }
            for i in 0..3 {
                let a = valid_point_id(tri[i], n).unwrap();
                let b = valid_point_id(tri[(i + 1) % 3], n).unwrap();
                if a != b {
                    adj[a].push(b);
                    adj[b].push(a);
                }
            }
        }
    }
    let mut visited = vec![false; n];
    let mut components = 0usize;
    for start in 0..n {
        if !used_points[start] || visited[start] {
            continue;
        }
        let mut stack = vec![start];
        while let Some(v) = stack.pop() {
            if visited[v] {
                continue;
            }
            visited[v] = true;
            for &next in &adj[v] {
                stack.push(next);
            }
        }
        components += 1;
    }
    components
}

fn used_polygon_points(mesh: &PolyData) -> Vec<bool> {
    let n = mesh.points.len();
    let mut used = vec![false; n];
    for cell in mesh.polys.iter() {
        if !valid_polygon(cell, n) {
            continue;
        }
        for &point_id in cell {
            if let Some(point_id) = valid_point_id(point_id, n) {
                used[point_id] = true;
            }
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            if !valid_triangle(tri, n) {
                continue;
            }
            for &point_id in tri {
                if let Some(point_id) = valid_point_id(point_id, n) {
                    used[point_id] = true;
                }
            }
        }
    }
    used
}

fn count_boundary_loops(edges: &[(i64, i64)]) -> usize {
    let mut adj: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let mut visited = std::collections::HashSet::new();
    let mut loops = 0usize;
    for &(start, _) in edges {
        if visited.contains(&start) {
            continue;
        }
        let mut stack = vec![start];
        while let Some(v) = stack.pop() {
            if !visited.insert(v) {
                continue;
            }
            if let Some(neighbors) = adj.get(&v) {
                for &next in neighbors {
                    stack.push(next);
                }
            }
        }
        loops += 1;
    }
    loops
}

fn add_edge(
    a: i64,
    b: i64,
    npoints: usize,
    edges: &mut std::collections::HashSet<(i64, i64)>,
    edge_count: &mut std::collections::HashMap<(i64, i64), usize>,
) {
    let Some(a) = valid_point_id(a, npoints) else {
        return;
    };
    let Some(b) = valid_point_id(b, npoints) else {
        return;
    };
    let a = a as i64;
    let b = b as i64;
    let e = if a < b { (a, b) } else { (b, a) };
    edges.insert(e);
    *edge_count.entry(e).or_insert(0) += 1;
}

fn add_triangle_strip_edges(
    strip: &[i64],
    npoints: usize,
    edges: &mut std::collections::HashSet<(i64, i64)>,
    edge_count: &mut std::collections::HashMap<(i64, i64), usize>,
) {
    for tri in strip.windows(3) {
        if !valid_triangle(tri, npoints) {
            continue;
        }
        add_edge(tri[0], tri[1], npoints, edges, edge_count);
        add_edge(tri[1], tri[2], npoints, edges, edge_count);
        add_edge(tri[2], tri[0], npoints, edges, edge_count);
    }
}

fn valid_triangle(tri: &[i64], npoints: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && tri
            .iter()
            .all(|&point_id| valid_point_id(point_id, npoints).is_some())
}

fn valid_polygon(cell: &[i64], npoints: usize) -> bool {
    cell.len() >= 3
        && cell
            .iter()
            .all(|&point_id| valid_point_id(point_id, npoints).is_some())
        && cell.windows(2).all(|edge| edge[0] != edge[1])
        && cell.first() != cell.last()
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_genus() {
        // Simple triangle: V=3, E=3, F=1 => chi=1 => genus=0 (or 1 for boundary)
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let chi = euler_characteristic(&mesh);
        assert_eq!(chi, 1); // open surface
    }

    #[test]
    fn test_open_triangle_genus() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(genus(&mesh), 0);
    }
}
