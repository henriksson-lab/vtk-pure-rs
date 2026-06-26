//! Harmonic UV parameterization for disk-topology meshes.
//!
//! Solves the Laplace equation on the mesh with boundary vertices
//! mapped to a circle, producing smooth UV coordinates.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute harmonic UV parameterization.
///
/// Boundary vertices are mapped to a unit circle, interior vertices
/// are solved via iterative Laplace relaxation.
pub fn harmonic_parameterize(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }

    let adj = build_adj(mesh, n);
    let boundary = find_boundary_loop(mesh, n);

    if boundary.is_empty() {
        return mesh.clone();
    }

    // Map boundary to unit circle
    let mut u = vec![0.5f64; n];
    let mut v = vec![0.5f64; n];
    let mut is_boundary = vec![false; n];

    for (i, &vi) in boundary.iter().enumerate() {
        let t = 2.0 * std::f64::consts::PI * i as f64 / boundary.len() as f64;
        u[vi] = 0.5 + 0.5 * t.cos();
        v[vi] = 0.5 + 0.5 * t.sin();
        is_boundary[vi] = true;
    }

    // Jacobi relaxation for interior vertices.
    for _ in 0..iterations {
        let mut next_u = u.clone();
        let mut next_v = v.clone();
        for i in 0..n {
            if is_boundary[i] || adj[i].is_empty() {
                continue;
            }
            let mut su = 0.0;
            let mut sv = 0.0;
            for &j in &adj[i] {
                su += u[j];
                sv += v[j];
            }
            let k = adj[i].len() as f64;
            next_u[i] = su / k;
            next_v[i] = sv / k;
        }
        u = next_u;
        v = next_v;
    }

    let mut tcoords = Vec::with_capacity(n * 2);
    for i in 0..n {
        tcoords.push(u[i]);
        tcoords.push(v[i]);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            adj[a].insert(b);
            adj[b].insert(a);
        }
    }
    adj.into_iter()
        .map(|s| {
            let mut ids: Vec<_> = s.into_iter().collect();
            ids.sort_unstable();
            ids
        })
        .collect()
}

fn find_boundary_loop(mesh: &PolyData, n: usize) -> Vec<usize> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }

    let mut boundary_adj: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }

    if boundary_adj.is_empty() {
        return Vec::new();
    }

    for neighbors in boundary_adj.values_mut() {
        neighbors.sort_unstable();
    }

    // Trace boundary loop
    let start = *boundary_adj.keys().min().unwrap();
    let mut loop_verts = vec![start];
    let mut visited = std::collections::HashSet::new();
    visited.insert(start);
    let mut current = start;

    loop {
        let next = boundary_adj
            .get(&current)
            .and_then(|nb| nb.iter().find(|&&v| !visited.contains(&v)));
        match next {
            Some(&v) => {
                loop_verts.push(v);
                visited.insert(v);
                current = v;
            }
            None => break,
        }
    }
    loop_verts
}

fn valid_point_id(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_grid() {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = harmonic_parameterize(&mesh, 50);
        let tc = result.point_data().tcoords();
        assert!(tc.is_some());
        let tc = tc.unwrap();
        assert_eq!(tc.num_tuples(), 25);
        // All UVs should be in [0,1]
        let mut buf = [0.0f64; 2];
        for i in 0..tc.num_tuples() {
            tc.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= -0.01 && buf[0] <= 1.01, "u={}", buf[0]);
            assert!(buf[1] >= -0.01 && buf[1] <= 1.01, "v={}", buf[1]);
        }
    }

    #[test]
    fn closed_mesh_no_boundary() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let result = harmonic_parameterize(&mesh, 10);
        // Should return clone since no boundary
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn first_iteration_uses_previous_values_for_all_interiors() {
        let mut pts = Vec::new();
        for y in 0..3 {
            for x in 0..4 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..2 {
            for x in 0..3 {
                let bl = y * 4 + x;
                tris.push([bl, bl + 1, bl + 5]);
                tris.push([bl, bl + 5, bl + 4]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let n = mesh.points.len();
        let adj = build_adj(&mesh, n);
        let boundary = find_boundary_loop(&mesh, n);
        let mut expected_u = vec![0.5f64; n];
        let mut expected_v = vec![0.5f64; n];
        let mut is_boundary = vec![false; n];
        for (i, &vi) in boundary.iter().enumerate() {
            let t = 2.0 * std::f64::consts::PI * i as f64 / boundary.len() as f64;
            expected_u[vi] = 0.5 + 0.5 * t.cos();
            expected_v[vi] = 0.5 + 0.5 * t.sin();
            is_boundary[vi] = true;
        }
        for i in 0..n {
            if is_boundary[i] || adj[i].is_empty() {
                continue;
            }
            expected_u[i] = adj[i]
                .iter()
                .map(|&j| if is_boundary[j] { expected_u[j] } else { 0.5 })
                .sum::<f64>()
                / adj[i].len() as f64;
            expected_v[i] = adj[i]
                .iter()
                .map(|&j| if is_boundary[j] { expected_v[j] } else { 0.5 })
                .sum::<f64>()
                / adj[i].len() as f64;
        }

        let result = harmonic_parameterize(&mesh, 1);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        for i in 0..n {
            if is_boundary[i] {
                continue;
            }
            tc.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - expected_u[i]).abs() < 1e-12, "u[{i}]");
            assert!((buf[1] - expected_v[i]).abs() < 1e-12, "v[{i}]");
        }
    }

    #[test]
    fn malformed_cells_do_not_panic() {
        let mut mesh =
            PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.polys.push_cell(&[0, -1, 99]);

        let result = harmonic_parameterize(&mesh, 5);
        assert_eq!(result.points.len(), 3);
    }
}
