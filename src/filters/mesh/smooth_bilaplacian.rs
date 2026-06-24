use std::collections::HashSet;

use crate::data::PolyData;

/// Bilaplacian (implicit fairing) smoothing of a PolyData mesh.
///
/// Applies Laplacian smoothing twice per iteration: the first pass computes
/// the Laplacian displacement, and the second pass smooths those displacements,
/// yielding a bilaplacian (fourth-order) effect. This produces smoother results
/// than standard Laplacian smoothing, with less volume shrinkage.
///
/// `lambda` controls the step size (0.0 = no movement, 1.0 = full step).
pub fn smooth_bilaplacian(input: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let mut output = input.clone();
    let n: usize = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    // Build adjacency
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

    let factor: f64 = lambda.clamp(0.0, 1.0);

    for _ in 0..iterations {
        // First Laplacian pass: compute L(x) for each vertex
        let lap1 = compute_laplacian(&output, &neighbors, n);

        // Second Laplacian pass: compute L(L(x)) by applying Laplacian to the
        // displacement vectors from the first pass
        let lap2 = compute_laplacian_of_vectors(&lap1, &neighbors, n);

        // Update positions: x' = x - lambda * L(L(x))
        for i in 0..n {
            let p = output.points.get(i);
            output.points.set(
                i,
                [
                    p[0] - factor * lap2[i][0],
                    p[1] - factor * lap2[i][1],
                    p[2] - factor * lap2[i][2],
                ],
            );
        }
    }

    output
}

/// Compute Laplacian displacement for each vertex: L(p_i) = avg(neighbors) - p_i
fn compute_laplacian(pd: &PolyData, neighbors: &[HashSet<usize>], n: usize) -> Vec<[f64; 3]> {
    let mut lap: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];

    for i in 0..n {
        let nbrs = &neighbors[i];
        if nbrs.is_empty() {
            continue;
        }
        let p = pd.points.get(i);
        let count: f64 = nbrs.len() as f64;
        let mut avg: [f64; 3] = [0.0, 0.0, 0.0];
        for &nb in nbrs {
            let q = pd.points.get(nb);
            avg[0] += q[0];
            avg[1] += q[1];
            avg[2] += q[2];
        }
        avg[0] /= count;
        avg[1] /= count;
        avg[2] /= count;

        lap[i] = [avg[0] - p[0], avg[1] - p[1], avg[2] - p[2]];
    }

    lap
}

/// Compute Laplacian of a vector field defined at vertices.
fn compute_laplacian_of_vectors(
    field: &[[f64; 3]],
    neighbors: &[HashSet<usize>],
    n: usize,
) -> Vec<[f64; 3]> {
    let mut lap: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];

    for i in 0..n {
        let nbrs = &neighbors[i];
        if nbrs.is_empty() {
            continue;
        }
        let fi = field[i];
        let count: f64 = nbrs.len() as f64;
        let mut avg: [f64; 3] = [0.0, 0.0, 0.0];
        for &nb in nbrs {
            let fj = field[nb];
            avg[0] += fj[0];
            avg[1] += fj[1];
            avg[2] += fj[2];
        }
        avg[0] /= count;
        avg[1] /= count;
        avg[2] /= count;

        lap[i] = [avg[0] - fi[0], avg[1] - fi[1], avg[2] - fi[2]];
    }

    lap
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diamond() -> PolyData {
        // A flat diamond shape: center vertex surrounded by 4 outer vertices
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[0, 3, 4]);
        pd.polys.push_cell(&[0, 4, 1]);
        pd
    }

    #[test]
    fn zero_iterations_returns_clone() {
        let input = make_diamond();
        let result = smooth_bilaplacian(&input, 0, 0.5);
        for i in 0..input.points.len() {
            let a = input.points.get(i);
            let b = result.points.get(i);
            assert!((a[0] - b[0]).abs() < 1e-12);
            assert!((a[1] - b[1]).abs() < 1e-12);
            assert!((a[2] - b[2]).abs() < 1e-12);
        }
    }

    #[test]
    fn smoothing_preserves_point_count() {
        let input = make_diamond();
        let result = smooth_bilaplacian(&input, 5, 0.5);
        assert_eq!(result.points.len(), input.points.len());
        assert_eq!(result.polys.num_cells(), input.polys.num_cells());
    }

    #[test]
    fn smoothing_moves_vertices() {
        // Perturb center vertex out of plane and verify smoothing pulls it back
        let mut input = make_diamond();
        input.points.set(0, [0.0, 0.0, 1.0]);

        let result = smooth_bilaplacian(&input, 10, 0.8);
        let center_z: f64 = result.points.get(0)[2];

        // After bilaplacian smoothing, center should have moved toward the plane
        assert!(
            center_z.abs() < 1.0,
            "center_z = {center_z}, expected closer to 0"
        );
    }
}
