use std::collections::{HashMap, HashSet};

use crate::data::PolyData;

/// Simple Laplacian smoothing of a PolyData mesh.
///
/// Each non-boundary vertex is moved toward the average of its neighbors by
/// a factor `lambda` (0.0 = no movement, 1.0 = move to average). The process
/// is repeated for the given number of `iterations`. Boundary vertices are
/// preserved (not moved).
pub fn smooth_laplacian_simple(input: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let mut output = input.clone();
    let n: usize = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    // Build adjacency
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            if a < n && b < n {
                neighbors[a].insert(b);
                neighbors[b].insert(a);
            }
        }
    }

    // Find boundary vertices
    let boundary = find_boundary_vertices(input);

    let factor: f64 = lambda.clamp(0.0, 1.0);

    for _ in 0..iterations {
        let mut new_positions: Vec<[f64; 3]> = Vec::with_capacity(n);

        for (i, nbrs) in neighbors.iter().enumerate() {
            let p = output.points.get(i);
            if boundary.contains(&i) || nbrs.is_empty() {
                new_positions.push(p);
                continue;
            }

            let mut avg = [0.0f64; 3];
            let count: f64 = nbrs.len() as f64;
            for &nb in nbrs {
                let q = output.points.get(nb);
                avg[0] += q[0];
                avg[1] += q[1];
                avg[2] += q[2];
            }
            avg[0] /= count;
            avg[1] /= count;
            avg[2] /= count;

            new_positions.push([
                p[0] + factor * (avg[0] - p[0]),
                p[1] + factor * (avg[1] - p[1]),
                p[2] + factor * (avg[2] - p[2]),
            ]);
        }

        for (i, &pos) in new_positions.iter().enumerate() {
            output.points.set(i, pos);
        }
    }

    output
}

fn find_boundary_vertices(input: &PolyData) -> HashSet<usize> {
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            if a >= input.points.len() || b >= input.points.len() {
                continue;
            }
            let edge = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    }

    let mut boundary = HashSet::new();
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh() -> PolyData {
        // A small mesh: center vertex (4) surrounded by 4 triangles
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.5, 1.5, 0.0], // slightly off-center
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
    }

    #[test]
    fn smoothing_moves_interior_toward_average() {
        let pd = make_mesh();
        let result = smooth_laplacian_simple(&pd, 1, 1.0);
        // Vertex 4 should move toward (0+2+2+0)/4=1.0, (0+0+2+2)/4=1.0
        let p = result.points.get(4);
        assert!((p[0] - 1.0).abs() < 1e-10, "x should be 1.0, got {}", p[0]);
        assert!((p[1] - 1.0).abs() < 1e-10, "y should be 1.0, got {}", p[1]);
    }

    #[test]
    fn boundary_vertices_preserved() {
        let pd = make_mesh();
        let result = smooth_laplacian_simple(&pd, 10, 1.0);
        // Corner vertices are boundary, should not move
        let p0 = result.points.get(0);
        assert_eq!(p0, [0.0, 0.0, 0.0]);
        let p1 = result.points.get(1);
        assert_eq!(p1, [2.0, 0.0, 0.0]);
    }

    #[test]
    fn zero_iterations_no_change() {
        let pd = make_mesh();
        let result = smooth_laplacian_simple(&pd, 0, 1.0);
        let p = result.points.get(4);
        assert_eq!(p, [1.5, 1.5, 0.0]);
    }
}
