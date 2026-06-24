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

    // Iterative Laplace relaxation for interior vertices
    for _ in 0..iterations {
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
            u[i] = su / k;
            v[i] = sv / k;
        }
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

fn find_boundary_loop(mesh: &PolyData, n: usize) -> Vec<usize> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
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

    // Trace boundary loop
    let start = *boundary_adj.keys().next().unwrap();
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
}
