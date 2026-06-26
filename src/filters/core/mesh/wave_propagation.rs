//! Wave propagation simulation on mesh connectivity.

use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

/// Simulate wave propagation from source vertices.
///
/// Uses the wave equation: u_tt = c² * Laplacian(u) with damping.
pub fn wave_propagate(
    mesh: &PolyData,
    sources: &[(usize, f64)],
    wave_speed: f64,
    damping: f64,
    dt: f64,
    steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut u = vec![0.0f64; n];
    let mut u_prev = vec![0.0f64; n];
    for &(si, amp) in sources {
        if si < n {
            u[si] = amp;
            u_prev[si] = amp;
        }
    }

    let c2 = wave_speed * wave_speed;
    for _ in 0..steps {
        let mut u_next = vec![0.0f64; n];
        for i in 0..n {
            if adj[i].is_empty() {
                u_next[i] = u[i];
                continue;
            }
            let lap: f64 = adj[i].iter().map(|&j| u[j] - u[i]).sum::<f64>() / adj[i].len() as f64;
            u_next[i] =
                2.0 * u[i] - u_prev[i] + dt * dt * c2 * lap - damping * dt * (u[i] - u_prev[i]);
        }
        u_prev = u;
        u = u_next;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WaveAmplitude",
            u,
            1,
        )));
    result
}

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in m.polys.iter() {
        add_closed_cell_edges(cell, n, &mut adj);
    }
    for cell in m.strips.iter() {
        add_triangle_strip_edges(cell, n, &mut adj);
    }
    for cell in m.lines.iter() {
        add_open_cell_edges(cell, n, &mut adj);
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn add_closed_cell_edges(cell: &[i64], n: usize, adj: &mut [HashSet<usize>]) {
    let nc = cell.len();
    if nc < 2 {
        return;
    }
    for i in 0..nc {
        add_adjacent_edge(cell[i], cell[(i + 1) % nc], n, adj);
    }
}

fn add_open_cell_edges(cell: &[i64], n: usize, adj: &mut [HashSet<usize>]) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..(cell.len() - 1) {
        add_adjacent_edge(cell[i], cell[i + 1], n, adj);
    }
}

fn add_triangle_strip_edges(cell: &[i64], n: usize, adj: &mut [HashSet<usize>]) {
    if cell.len() < 3 {
        return;
    }
    for i in 0..(cell.len() - 2) {
        add_adjacent_edge(cell[i], cell[i + 1], n, adj);
        add_adjacent_edge(cell[i + 1], cell[i + 2], n, adj);
        add_adjacent_edge(cell[i + 2], cell[i], n, adj);
    }
}

fn add_adjacent_edge(a: i64, b: i64, n: usize, adj: &mut [HashSet<usize>]) {
    let Some(x) = point_id(a, n) else {
        return;
    };
    let Some(y) = point_id(b, n) else {
        return;
    };
    if x != y {
        adj[x].insert(y);
        adj[y].insert(x);
    }
}

fn point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wave() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = wave_propagate(&mesh, &[(50, 1.0)], 1.0, 0.01, 0.1, 20);
        let arr = result.point_data().get_array("WaveAmplitude").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(50, &mut buf);
        assert!(buf[0].abs() > 0.0);
    }

    #[test]
    fn isolated_source_keeps_amplitude() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let result = wave_propagate(&mesh, &[(0, 1.0)], 1.0, 0.01, 0.1, 20);
        let arr = result.point_data().get_array("WaveAmplitude").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
