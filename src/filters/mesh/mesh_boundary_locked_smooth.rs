//! Laplacian smoothing that keeps boundary vertices fixed.
use crate::data::{Points, PolyData};

pub fn boundary_locked_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        add_polygon_edges(cell, n, &mut adj, &mut edge_count);
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(strip, n, &mut adj, &mut edge_count);
    }
    let mut boundary = vec![false; n];
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary[a] = true;
            boundary[b] = true;
        }
    }
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();
    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if boundary[i] || adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f64;
            for d in 0..3 {
                let avg: f64 = adj[i].iter().map(|&j| positions[j][d]).sum::<f64>() / k;
                new_pos[i][d] += lambda * (avg - positions[i][d]);
            }
        }
        positions = new_pos;
    }
    let mut pts = Points::<f64>::new();
    for p in &positions {
        pts.push(*p);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

fn add_polygon_edges(
    cell: &[i64],
    npoints: usize,
    adj: &mut [Vec<usize>],
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    let nc = cell.len();
    if nc < 2 || !valid_cell(cell, npoints) {
        return;
    }
    for i in 0..nc {
        add_edge(
            cell[i] as usize,
            cell[(i + 1) % nc] as usize,
            adj,
            edge_count,
        );
    }
}

fn add_triangle_strip_edges(
    strip: &[i64],
    npoints: usize,
    adj: &mut [Vec<usize>],
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    if strip.len() < 3 || !valid_cell(strip, npoints) {
        return;
    }
    for i in 0..strip.len() - 2 {
        let tri = if i % 2 == 0 {
            [strip[i], strip[i + 1], strip[i + 2]]
        } else {
            [strip[i + 1], strip[i], strip[i + 2]]
        };
        add_edge(tri[0] as usize, tri[1] as usize, adj, edge_count);
        add_edge(tri[1] as usize, tri[2] as usize, adj, edge_count);
        add_edge(tri[2] as usize, tri[0] as usize, adj, edge_count);
    }
}

fn add_edge(
    a: usize,
    b: usize,
    adj: &mut [Vec<usize>],
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    if a == b {
        return;
    }
    if !adj[a].contains(&b) {
        adj[a].push(b);
    }
    if !adj[b].contains(&a) {
        adj[b].push(a);
    }
    let e = if a < b { (a, b) } else { (b, a) };
    *edge_count.entry(e).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_locked_smooth() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.8, 0.1],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = boundary_locked_smooth(&mesh, 5, 0.5);
        // Boundary vertices should not move
        let p0 = r.points.get(0);
        assert_eq!(p0, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn triangle_strip_boundary_vertices_are_locked() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let r = boundary_locked_smooth(&mesh, 3, 1.0);

        for i in 0..4 {
            assert_eq!(r.points.get(i), mesh.points.get(i));
        }
    }
}
