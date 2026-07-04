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
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            if a == b {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            if !valid_triangle(tri, n) {
                continue;
            }
            add_neighbor_edge(tri[0], tri[1], n, &mut neighbors);
            add_neighbor_edge(tri[1], tri[2], n, &mut neighbors);
            add_neighbor_edge(tri[2], tri[0], n, &mut neighbors);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_edge(edge[0], edge[1], n, &mut neighbors);
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
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("HKS", hks_data, nt)));
    result.point_data_mut().set_active_scalars("HKS");
    result
}

/// Compute auto-diffusion: how much heat stays at each vertex after time t.
pub fn auto_diffusion(mesh: &PolyData, time: f64, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            if a == b {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            if !valid_triangle(tri, n) {
                continue;
            }
            add_neighbor_edge(tri[0], tri[1], n, &mut neighbors);
            add_neighbor_edge(tri[1], tri[2], n, &mut neighbors);
            add_neighbor_edge(tri[2], tri[0], n, &mut neighbors);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_edge(edge[0], edge[1], n, &mut neighbors);
        }
    }

    let num_iterations = iterations.max(1);
    let dt = time / num_iterations as f64;
    let mut diffusion = vec![0.0f64; n];

    for vi in 0..n {
        let mut heat = vec![0.0f64; n];
        heat[vi] = 1.0;
        for _ in 0..num_iterations {
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
        diffusion[vi] = heat[vi];
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Diffusion",
            diffusion,
            1,
        )));
    result.point_data_mut().set_active_scalars("Diffusion");
    result
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && tri
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn add_neighbor_edge(a: i64, b: i64, n_points: usize, neighbors: &mut [Vec<usize>]) {
    let Some(a) = valid_point_id(a, n_points) else {
        return;
    };
    let Some(b) = valid_point_id(b, n_points) else {
        return;
    };
    if a == b {
        return;
    }
    if !neighbors[a].contains(&b) {
        neighbors[a].push(b);
    }
    if !neighbors[b].contains(&a) {
        neighbors[b].push(a);
    }
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
        let arr = r.point_data().get_array("HKS").unwrap();
        assert_eq!(arr.num_components(), 2);
        assert_eq!(arr.num_tuples(), mesh.points.len());
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

    #[test]
    fn test_diffusion_is_diagonal_response() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let r = auto_diffusion(&mesh, 0.5, 1);
        let arr = r.point_data().get_array("Diffusion").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] < 1.0);
    }
}
