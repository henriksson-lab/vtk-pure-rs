use std::collections::HashSet;

use crate::data::PolyData;

/// Extract a curve skeleton from a mesh by iterative Laplacian contraction.
///
/// The algorithm repeatedly applies Laplacian smoothing with a strong contraction
/// weight, causing the mesh surface to collapse toward its medial axis. After
/// enough iterations the mesh approximates a 1D curve skeleton.
///
/// `iterations` controls the number of contraction passes.
/// `contraction_weight` controls how strongly vertices move toward the average
/// of their neighbors each iteration (clamped to [0, 1]).
///
/// The output is a PolyData with the same connectivity as the input but with
/// contracted vertex positions.
pub fn extract_skeleton(input: &PolyData, iterations: usize, contraction_weight: f64) -> PolyData {
    let mut output = input.clone();
    let n: usize = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    let weight: f64 = contraction_weight.clamp(0.0, 1.0);

    // Build adjacency from polygon connectivity
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        let len: usize = cell.len();
        for j in 0..len {
            let a: usize = cell[j] as usize;
            let b: usize = cell[(j + 1) % len] as usize;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    // Also include edges from lines if present
    for cell in input.lines.iter() {
        for j in 0..(cell.len().saturating_sub(1)) {
            let a: usize = cell[j] as usize;
            let b: usize = cell[j + 1] as usize;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    for _ in 0..iterations {
        let mut new_positions: Vec<[f64; 3]> = Vec::with_capacity(n);

        for i in 0..n {
            let p: [f64; 3] = output.points.get(i);
            let nbrs = &neighbors[i];

            if nbrs.is_empty() {
                new_positions.push(p);
                continue;
            }

            // Compute centroid of neighbors
            let count: f64 = nbrs.len() as f64;
            let mut avg: [f64; 3] = [0.0, 0.0, 0.0];
            for &nb in nbrs {
                let q: [f64; 3] = output.points.get(nb);
                avg[0] += q[0];
                avg[1] += q[1];
                avg[2] += q[2];
            }
            avg[0] /= count;
            avg[1] /= count;
            avg[2] /= count;

            // Move strongly toward centroid (contraction)
            new_positions.push([
                p[0] + weight * (avg[0] - p[0]),
                p[1] + weight * (avg[1] - p[1]),
                p[2] + weight * (avg[2] - p[2]),
            ]);
        }

        for i in 0..n {
            output.points.set(i, new_positions[i]);
        }
    }

    output
}

/// Compute the total edge length of a PolyData mesh (sum of all polygon edge lengths).
#[cfg(test)]
fn total_edge_length(pd: &PolyData) -> f64 {
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut total: f64 = 0.0;

    for cell in pd.polys.iter() {
        let len: usize = cell.len();
        for j in 0..len {
            let a: usize = cell[j] as usize;
            let b: usize = cell[(j + 1) % len] as usize;
            let key: (usize, usize) = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa: [f64; 3] = pd.points.get(a);
                let pb: [f64; 3] = pd.points.get(b);
                let dx: f64 = pa[0] - pb[0];
                let dy: f64 = pa[1] - pb[1];
                let dz: f64 = pa[2] - pb[2];
                total += (dx * dx + dy * dy + dz * dz).sqrt();
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_triangle_mesh() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd
    }

    fn make_box_like_mesh() -> PolyData {
        // A simple mesh with 5 vertices (4 corners + center)
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 4]);
        pd.polys.push_cell(&[1, 2, 4]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[3, 0, 4]);
        pd
    }

    #[test]
    fn zero_iterations_returns_clone() {
        let mesh = make_triangle_mesh();
        let result = extract_skeleton(&mesh, 0, 0.9);
        for i in 0..mesh.points.len() {
            let p: [f64; 3] = mesh.points.get(i);
            let q: [f64; 3] = result.points.get(i);
            assert!((p[0] - q[0]).abs() < 1e-10);
            assert!((p[1] - q[1]).abs() < 1e-10);
            assert!((p[2] - q[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn contraction_reduces_edge_length() {
        let mesh = make_box_like_mesh();
        let contracted = extract_skeleton(&mesh, 50, 0.9);
        let original_len: f64 = total_edge_length(&mesh);
        let contracted_len: f64 = total_edge_length(&contracted);
        assert!(contracted_len < original_len);
    }

    #[test]
    fn many_iterations_collapses_to_point() {
        let mesh = make_triangle_mesh();
        let result = extract_skeleton(&mesh, 500, 1.0);
        // All points should be very close together
        let p0: [f64; 3] = result.points.get(0);
        let p1: [f64; 3] = result.points.get(1);
        let p2: [f64; 3] = result.points.get(2);
        let d01: f64 =
            ((p0[0] - p1[0]).powi(2) + (p0[1] - p1[1]).powi(2) + (p0[2] - p1[2]).powi(2)).sqrt();
        let d02: f64 =
            ((p0[0] - p2[0]).powi(2) + (p0[1] - p2[1]).powi(2) + (p0[2] - p2[2]).powi(2)).sqrt();
        assert!(d01 < 0.01);
        assert!(d02 < 0.01);
    }
}
