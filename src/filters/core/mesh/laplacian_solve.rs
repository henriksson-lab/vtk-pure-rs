//! Solve Laplace/Poisson equations on mesh connectivity.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Solve the Laplace equation on a mesh with Dirichlet boundary conditions.
///
/// Fixed vertices have values set in `boundary_values` (index→value).
/// Interior vertices are solved via Jacobi iteration.
pub fn laplace_solve(
    mesh: &PolyData, boundary_values: &[(usize, f64)], iterations: usize, array_name: &str,
) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut values = vec![0.0f64; n];
    let mut fixed = vec![false; n];
    for &(idx, val) in boundary_values { if idx < n { values[idx] = val; fixed[idx] = true; } }

    for _ in 0..iterations {
        let mut new_vals = values.clone();
        for i in 0..n {
            if fixed[i] || adj[i].is_empty() { continue; }
            let sum: f64 = adj[i].iter().map(|&j| values[j]).sum();
            new_vals[i] = sum / adj[i].len() as f64;
        }
        values = new_vals;
    }

    let mut result = mesh.clone();
    result.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec(array_name, values, 1)));
    result
}

/// Solve a Poisson equation ∇²u = f with boundary conditions.
pub fn poisson_solve(
    mesh: &PolyData, source_array: &str, boundary_values: &[(usize, f64)], iterations: usize, result_name: &str,
) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let src = mesh.point_data().get_array(source_array);
    let mut values = vec![0.0f64; n];
    let mut fixed = vec![false; n];
    for &(idx, val) in boundary_values { if idx < n { values[idx] = val; fixed[idx] = true; } }
    let mut buf = [0.0f64];

    for _ in 0..iterations {
        let mut new_vals = values.clone();
        for i in 0..n {
            if fixed[i] || adj[i].is_empty() { continue; }
            let sum: f64 = adj[i].iter().map(|&j| values[j]).sum();
            let f_val = if let Some(s) = src {
                if i < s.num_tuples() { s.tuple_as_f64(i, &mut buf); buf[0] } else { 0.0 }
            } else { 0.0 };
            new_vals[i] = (sum + f_val) / adj[i].len() as f64;
        }
        values = new_vals;
    }

    let mut result = mesh.clone();
    result.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec(result_name, values, 1)));
    result
}

/// Compute the harmonic weight between two connected vertices.
pub fn harmonic_weights(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    // For each vertex, compute the sum of edge weights (1/distance)
    let mut weight_sum = vec![0.0f64; n];
    for i in 0..n {
        for &j in &adj[i] {
            let pi = mesh.points.get(i); let pj = mesh.points.get(j);
            let d = ((pi[0]-pj[0]).powi(2)+(pi[1]-pj[1]).powi(2)+(pi[2]-pj[2]).powi(2)).sqrt();
            if d > 1e-15 { weight_sum[i] += 1.0 / d; }
        }
    }
    let mut result = mesh.clone();
    result.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec("HarmonicWeight", weight_sum, 1)));
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() { let nc = cell.len(); for i in 0..nc {
        let a = cell[i] as usize; let b = cell[(i+1)%nc] as usize;
        if a<n&&b<n { adj[a].insert(b); adj[b].insert(a); }
    }}
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn laplace_1d() {
        // Line of 5 vertices: fix endpoints at 0 and 1
        let mesh = PolyData::from_triangles(
            vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[2.0,0.0,0.0],[3.0,0.0,0.0],[4.0,0.0,0.0],
                 [0.0,1.0,0.0],[1.0,1.0,0.0],[2.0,1.0,0.0],[3.0,1.0,0.0],[4.0,1.0,0.0]],
            vec![[0,1,6],[0,6,5],[1,2,7],[1,7,6],[2,3,8],[2,8,7],[3,4,9],[3,9,8]]);
        let result = laplace_solve(&mesh, &[(0,0.0),(4,1.0),(5,0.0),(9,1.0)], 100, "u");
        let arr = result.point_data().get_array("u").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 0.1, "midpoint should be ~0.5, got {}", buf[0]);
    }
    #[test]
    fn harmonic() {
        let mesh = PolyData::from_triangles(
            vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![[0,1,2]]);
        let result = harmonic_weights(&mesh);
        assert!(result.point_data().get_array("HarmonicWeight").is_some());
    }
}
