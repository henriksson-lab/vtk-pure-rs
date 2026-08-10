//! Elastic deformation: apply force fields and compute equilibrium shapes.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Apply a radial force field centered at a point.
pub fn radial_force_deform(
    mesh: &PolyData,
    center: [f64; 3],
    strength: f64,
    radius: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let r2 = radius * radius;

    for _ in 0..iterations {
        let mut forces = vec![[0.0; 3]; n];
        for i in 0..n {
            let d = [
                pos[i][0] - center[0],
                pos[i][1] - center[1],
                pos[i][2] - center[2],
            ];
            let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if d2 < r2 && d2 > 1e-15 {
                let dist = d2.sqrt();
                let falloff = 1.0 - dist / radius;
                let f = strength * falloff / dist;
                for c in 0..3 {
                    forces[i][c] += f * d[c];
                }
            }
            // Spring restoration
            for &j in &adj[i] {
                let dd = [
                    pos[j][0] - pos[i][0],
                    pos[j][1] - pos[i][1],
                    pos[j][2] - pos[i][2],
                ];
                for c in 0..3 {
                    forces[i][c] += 0.1 * dd[c];
                }
            }
        }
        for i in 0..n {
            for c in 0..3 {
                pos[i][c] += forces[i][c] * 0.01;
            }
        }
    }

    let disp: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((pos[i][0] - p[0]).powi(2) + (pos[i][1] - p[1]).powi(2) + (pos[i][2] - p[2]).powi(2))
                .sqrt()
        })
        .collect();

    let mut result = mesh.clone();
    result.points = Points::from(pos);
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Displacement",
            disp,
            1,
        )));
    result
}

/// Inflate/deflate a mesh along its vertex normals.
///
/// The single implementation lives in
/// [`crate::filters::mesh::mesh_scale_per_vertex`].
pub use crate::filters::mesh::mesh_scale_per_vertex::{deflate, inflate};

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for c in m.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let x = c[i] as usize;
            let y = c[(i + 1) % nc] as usize;
            if x < n && y < n {
                a[x].insert(y);
                a[y].insert(x);
            }
        }
    }
    a.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn radial() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let result = radial_force_deform(&mesh, [0.5, 0.0, 0.0], 1.0, 2.0, 10);
        assert!(result.point_data().get_array("Displacement").is_some());
    }
}
