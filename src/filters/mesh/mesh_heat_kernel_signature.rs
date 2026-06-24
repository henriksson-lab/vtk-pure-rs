//! Heat Kernel Signature (HKS) shape descriptor.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute a simplified Heat Kernel Signature for each vertex.
/// Uses diffusion on the mesh graph as an approximation.
pub fn heat_kernel_signature(mesh: &PolyData, time_steps: &[f64]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || time_steps.is_empty() {
        return mesh.clone();
    }

    // Build adjacency with cotangent weights (simplified: uniform weights)
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) {
                    neighbors[a].push(b);
                }
                if !neighbors[b].contains(&a) {
                    neighbors[b].push(a);
                }
            }
        }
    }

    // For each time scale, diffuse a delta function and record diagonal
    let nt = time_steps.len();
    let mut hks_data = vec![0.0f64; n * nt];

    for (ti, &t) in time_steps.iter().enumerate() {
        // Simple diffusion: iterate heat equation
        let iters = (t * 10.0).ceil() as usize;
        let dt = t / iters.max(1) as f64;

        for vi in 0..n {
            let mut heat = vec![0.0f64; n];
            heat[vi] = 1.0;
            for _ in 0..iters {
                let mut new_heat = heat.clone();
                for j in 0..n {
                    if neighbors[j].is_empty() {
                        continue;
                    }
                    let k = neighbors[j].len() as f64;
                    let lap: f64 = neighbors[j]
                        .iter()
                        .map(|&nb| heat[nb] - heat[j])
                        .sum::<f64>()
                        / k;
                    new_heat[j] += dt * lap;
                }
                heat = new_heat;
            }
            hks_data[vi * nt + ti] = heat[vi]; // diagonal of heat kernel
        }
    }

    let mut result = mesh.clone();
    // Store first time scale as scalar
    let first_t: Vec<f64> = (0..n).map(|i| hks_data[i * nt]).collect();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("HKS", first_t, 1)));
    result.point_data_mut().set_active_scalars("HKS");
    result
}

/// Compute auto-diffusion: how much heat stays at each vertex after time t.
pub fn auto_diffusion(mesh: &PolyData, time: f64, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) {
                    neighbors[a].push(b);
                }
                if !neighbors[b].contains(&a) {
                    neighbors[b].push(a);
                }
            }
        }
    }

    let dt = time / iterations.max(1) as f64;
    let mut heat: Vec<f64> = vec![1.0; n]; // uniform initial
    for _ in 0..iterations {
        let mut new_heat = heat.clone();
        for j in 0..n {
            if neighbors[j].is_empty() {
                continue;
            }
            let k = neighbors[j].len() as f64;
            let lap: f64 = neighbors[j]
                .iter()
                .map(|&nb| heat[nb] - heat[j])
                .sum::<f64>()
                / k;
            new_heat[j] += dt * lap;
        }
        heat = new_heat;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Diffusion", heat, 1)));
    result.point_data_mut().set_active_scalars("Diffusion");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hks() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = heat_kernel_signature(&mesh, &[0.1, 1.0]);
        assert!(r.point_data().get_array("HKS").is_some());
    }
    #[test]
    fn test_diffusion() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = auto_diffusion(&mesh, 1.0, 10);
        assert!(r.point_data().get_array("Diffusion").is_some());
    }
}
