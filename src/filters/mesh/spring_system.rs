//! Spring-mass system simulation on mesh connectivity.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Simulate a spring-mass system on a mesh.
///
/// Each edge acts as a spring. External forces and fixed vertices supported.
pub fn spring_simulate(
    mesh: &PolyData,
    external_force: [f64; 3],
    fixed_vertices: &[usize],
    stiffness: f64,
    damping: f64,
    dt: f64,
    steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);
    let fixed: std::collections::HashSet<usize> = fixed_vertices.iter().cloned().collect();

    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let rest: Vec<Vec<f64>> = (0..n)
        .map(|i| adj[i].iter().map(|&j| edge_len(&pos, i, j)).collect())
        .collect();
    let mut vel = vec![[0.0f64; 3]; n];

    for _ in 0..steps {
        let mut forces = vec![[0.0; 3]; n];
        for i in 0..n {
            for c in 0..3 {
                forces[i][c] += external_force[c];
            }
        }
        for i in 0..n {
            for (ni, &j) in adj[i].iter().enumerate() {
                let d = [
                    pos[j][0] - pos[i][0],
                    pos[j][1] - pos[i][1],
                    pos[j][2] - pos[i][2],
                ];
                let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let r = if ni < rest[i].len() { rest[i][ni] } else { 1.0 };
                if dist > 1e-15 {
                    let f = stiffness * (dist - r) / dist;
                    for c in 0..3 {
                        forces[i][c] += f * d[c];
                    }
                }
            }
        }
        for i in 0..n {
            if fixed.contains(&i) {
                continue;
            }
            for c in 0..3 {
                vel[i][c] = (vel[i][c] + forces[i][c] * dt) * (1.0 - damping);
                pos[i][c] += vel[i][c] * dt;
            }
        }
    }

    // Add displacement magnitude
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

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn edge_len(pts: &[[f64; 3]], a: usize, b: usize) -> f64 {
    ((pts[a][0] - pts[b][0]).powi(2)
        + (pts[a][1] - pts[b][1]).powi(2)
        + (pts[a][2] - pts[b][2]).powi(2))
    .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gravity_drop() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let result = spring_simulate(&mesh, [0.0, 0.0, -1.0], &[0], 10.0, 0.1, 0.01, 50);
        assert!(result.point_data().get_array("Displacement").is_some());
        // Non-fixed vertices should have moved
        let arr = result.point_data().get_array("Displacement").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(3, &mut buf);
        assert!(buf[0] > 0.0);
    }
}
