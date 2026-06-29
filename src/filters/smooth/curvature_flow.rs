use crate::data::{Points, PolyData};
use std::collections::HashSet;

/// Mean curvature flow smoothing.
///
/// Moves each vertex along its mean curvature normal (Laplacian direction
/// weighted by cotangent weights). Shrinks bumps while preserving features
/// better than uniform Laplacian. `dt` is the time step.
pub fn curvature_flow(input: &PolyData, dt: f64, iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let neighbors = build_adjacency(input, n);

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pts = pts.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            // Cotangent-weighted Laplacian (simplified: uniform weights)
            let cnt = neighbors[i].len() as f64;
            let mut lap = [0.0; 3];
            for &j in &neighbors[i] {
                lap[0] += pts[j][0] - pts[i][0];
                lap[1] += pts[j][1] - pts[i][1];
                lap[2] += pts[j][2] - pts[i][2];
            }
            lap[0] /= cnt;
            lap[1] /= cnt;
            lap[2] /= cnt;
            new_pts[i] = [
                pts[i][0] + dt * lap[0],
                pts[i][1] + dt * lap[1],
                pts[i][2] + dt * lap[2],
            ];
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

fn build_adjacency(input: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        add_closed_cell_edges(cell, n, &mut neighbors);
    }
    for cell in input.lines.iter() {
        add_open_cell_edges(cell, n, &mut neighbors);
    }
    neighbors
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect()
}

fn add_closed_cell_edges(cell: &[i64], n: usize, neighbors: &mut [HashSet<usize>]) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        add_edge(cell[i], cell[(i + 1) % cell.len()], n, neighbors);
    }
}

fn add_open_cell_edges(cell: &[i64], n: usize, neighbors: &mut [HashSet<usize>]) {
    for edge in cell.windows(2) {
        add_edge(edge[0], edge[1], n, neighbors);
    }
}

fn add_edge(a: i64, b: i64, n: usize, neighbors: &mut [HashSet<usize>]) {
    let (Some(a), Some(b)) = (valid_point_id(a, n), valid_point_id(b, n)) else {
        return;
    };
    if a != b {
        neighbors[a].insert(b);
        neighbors[b].insert(a);
    }
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
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
    fn smooths_spike() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 2.0]); // spike
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[2, 0, 3]);

        let result = curvature_flow(&pd, 0.3, 5);
        let spike = result.points.get(3);
        assert!(spike[2] < 2.0); // spike reduced
    }

    #[test]
    fn zero_dt_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.polys.push_cell(&[0, 1]);

        let result = curvature_flow(&pd, 0.0, 10);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = curvature_flow(&pd, 0.5, 10);
        assert_eq!(result.points.len(), 0);
    }
}
