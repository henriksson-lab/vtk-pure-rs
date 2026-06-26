//! Cloth simulation mesh source: a flat grid with spring properties.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate a cloth mesh (flat grid) with per-vertex mass and fixed-point markers.
pub fn cloth_mesh(width: f64, depth: f64, nx: usize, ny: usize) -> PolyData {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut mass = Vec::new();
    let mut fixed = Vec::new();

    for j in 0..=ny {
        for i in 0..=nx {
            let x = width * i as f64 / nx as f64 - width / 2.0;
            let y = depth * j as f64 / ny as f64 - depth / 2.0;
            points.push([x, y, 0.0]);
            mass.push(1.0);
            // Fix top edge
            fixed.push(if j == ny { 1.0 } else { 0.0 });
        }
    }

    let row = nx + 1;
    for j in 0..ny {
        for i in 0..nx {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Mass", mass, 1)));
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Fixed", fixed, 1)));
    mesh
}

/// Simple cloth simulation step: apply gravity and spring forces.
pub fn cloth_simulate_step(
    mesh: &PolyData,
    gravity: [f64; 3],
    stiffness: f64,
    damping: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let fixed_arr = mesh.point_data().get_array("Fixed");
    let adj = build_adj(mesh, n);

    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let rest_lengths: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            adj[i]
                .iter()
                .map(|&j| {
                    ((positions[i][0] - positions[j][0]).powi(2)
                        + (positions[i][1] - positions[j][1]).powi(2)
                        + (positions[i][2] - positions[j][2]).powi(2))
                    .sqrt()
                })
                .collect()
        })
        .collect();

    let mut velocities = vec![[0.0f64; 3]; n];
    let dt = 0.01;

    for _ in 0..10 {
        // sub-steps
        let mut forces = vec![[0.0; 3]; n];
        // Gravity
        for i in 0..n {
            for c in 0..3 {
                forces[i][c] += gravity[c];
            }
        }

        // Springs
        for i in 0..n {
            for (ni, &j) in adj[i].iter().enumerate() {
                let dx = [
                    positions[j][0] - positions[i][0],
                    positions[j][1] - positions[i][1],
                    positions[j][2] - positions[i][2],
                ];
                let dist = (dx[0] * dx[0] + dx[1] * dx[1] + dx[2] * dx[2]).sqrt();
                let rest = if ni < rest_lengths[i].len() {
                    rest_lengths[i][ni]
                } else {
                    1.0
                };
                if dist > 1e-15 {
                    let f = stiffness * (dist - rest) / dist;
                    for c in 0..3 {
                        forces[i][c] += f * dx[c];
                    }
                }
            }
        }

        // Integrate
        let mut buf = [0.0f64];
        for i in 0..n {
            let is_fixed = if let Some(fa) = fixed_arr {
                fa.tuple_as_f64(i, &mut buf);
                buf[0] > 0.5
            } else {
                false
            };
            if is_fixed {
                continue;
            }
            for c in 0..3 {
                velocities[i][c] = (velocities[i][c] + forces[i][c] * dt) * (1.0 - damping);
                positions[i][c] += velocities[i][c] * dt;
            }
        }
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_cloth() {
        let cloth = cloth_mesh(2.0, 2.0, 8, 8);
        assert_eq!(cloth.points.len(), 81); // 9x9
        assert!(cloth.point_data().get_array("Mass").is_some());
        assert!(cloth.point_data().get_array("Fixed").is_some());
    }

    #[test]
    fn simulate_step() {
        let cloth = cloth_mesh(1.0, 1.0, 4, 4);
        let result = cloth_simulate_step(&cloth, [0.0, 0.0, -9.8], 50.0, 0.1);
        assert_eq!(result.points.len(), cloth.points.len());
        // Non-fixed points should have moved downward
        let p = result.points.get(0); // bottom-left, not fixed
        assert!(p[2] < 0.0, "should droop under gravity");
    }

    #[test]
    fn fixed_points_stay() {
        let cloth = cloth_mesh(1.0, 1.0, 4, 4);
        let result = cloth_simulate_step(&cloth, [0.0, 0.0, -9.8], 50.0, 0.1);
        // Top edge should stay at z=0
        let top_idx = 4 * 5 + 2; // row 4 (top), col 2
        let p = result.points.get(top_idx);
        assert!((p[2] - 0.0).abs() < 1e-10, "fixed point moved: z={}", p[2]);
    }
}
