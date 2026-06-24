//! UV atlas unwrapping: partition mesh into charts and flatten to UV space.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Unwrap a mesh into UV space using angle-based flattening.
///
/// Each connected component gets its own UV island. Works best on
/// disk-topology patches.
pub fn unwrap_to_uv_atlas(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }

    let adj = build_adj(mesh, n);

    // Find connected components
    let mut comp = vec![usize::MAX; n];
    let mut next_comp = 0;
    for seed in 0..n {
        if comp[seed] != usize::MAX || adj[seed].is_empty() {
            continue;
        }
        let mut q = vec![seed];
        comp[seed] = next_comp;
        while let Some(v) = q.pop() {
            for &nb in &adj[v] {
                if comp[nb] == usize::MAX {
                    comp[nb] = next_comp;
                    q.push(nb);
                }
            }
        }
        next_comp += 1;
    }

    // For each component, flatten to UV using Tutte-style embedding
    let mut u_data = vec![0.0f64; n];
    let mut v_data = vec![0.0f64; n];

    for ci in 0..next_comp {
        let verts: Vec<usize> = (0..n).filter(|&i| comp[i] == ci).collect();
        if verts.len() < 3 {
            continue;
        }

        // Find boundary vertices for this component
        let boundary = find_component_boundary(mesh, &verts);

        if boundary.is_empty() {
            // No boundary — project to PCA plane
            pca_flatten(mesh, &verts, &mut u_data, &mut v_data);
        } else {
            // Map boundary to circle, solve Tutte for interior
            circle_boundary_tutte(mesh, &verts, &boundary, &adj, &mut u_data, &mut v_data);
        }

        // Offset UV islands to avoid overlap
        let u_offset = ci as f64 * 1.2;
        for &vi in &verts {
            u_data[vi] += u_offset;
        }
    }

    let mut tcoords = Vec::with_capacity(n * 2);
    for i in 0..n {
        tcoords.push(u_data[i]);
        tcoords.push(v_data[i]);
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

fn pca_flatten(mesh: &PolyData, verts: &[usize], u: &mut [f64], v: &mut [f64]) {
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for &vi in verts {
        let p = mesh.points.get(vi);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = verts.len() as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    // Use first two principal axes for UV
    // Simplified: project onto XY plane relative to centroid
    for &vi in verts {
        let p = mesh.points.get(vi);
        u[vi] = p[0] - cx;
        v[vi] = p[1] - cy;
    }
    // Normalize to [0,1]
    let u_min = verts.iter().map(|&i| u[i]).fold(f64::MAX, f64::min);
    let u_max = verts.iter().map(|&i| u[i]).fold(f64::MIN, f64::max);
    let v_min = verts.iter().map(|&i| v[i]).fold(f64::MAX, f64::min);
    let v_max = verts.iter().map(|&i| v[i]).fold(f64::MIN, f64::max);
    let ur = (u_max - u_min).max(1e-15);
    let vr = (v_max - v_min).max(1e-15);
    for &vi in verts {
        u[vi] = (u[vi] - u_min) / ur;
        v[vi] = (v[vi] - v_min) / vr;
    }
}

fn circle_boundary_tutte(
    mesh: &PolyData,
    verts: &[usize],
    boundary: &[usize],
    adj: &[Vec<usize>],
    u: &mut [f64],
    v: &mut [f64],
) {
    let nb = boundary.len();
    let is_boundary: std::collections::HashSet<usize> = boundary.iter().cloned().collect();

    // Map boundary to unit circle
    for (i, &bi) in boundary.iter().enumerate() {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / nb as f64;
        u[bi] = 0.5 + 0.5 * angle.cos();
        v[bi] = 0.5 + 0.5 * angle.sin();
    }

    // Interior vertices: iteratively solve Tutte (average of neighbors)
    let interior: Vec<usize> = verts
        .iter()
        .filter(|&&vi| !is_boundary.contains(&vi))
        .cloned()
        .collect();
    for _ in 0..50 {
        for &vi in &interior {
            let neighbors: Vec<usize> = adj[vi]
                .iter()
                .filter(|&&nb| verts.contains(&nb))
                .cloned()
                .collect();
            if neighbors.is_empty() {
                continue;
            }
            let k = neighbors.len() as f64;
            u[vi] = neighbors.iter().map(|&nb| u[nb]).sum::<f64>() / k;
            v[vi] = neighbors.iter().map(|&nb| v[nb]).sum::<f64>() / k;
        }
    }
}

fn find_component_boundary(mesh: &PolyData, verts: &[usize]) -> Vec<usize> {
    let vert_set: std::collections::HashSet<usize> = verts.iter().cloned().collect();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        let in_comp = cell.iter().all(|&pid| vert_set.contains(&(pid as usize)));
        if !in_comp {
            continue;
        }
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut bnd_verts: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            bnd_verts.insert(a);
            bnd_verts.insert(b);
        }
    }

    // Order boundary into a loop
    if bnd_verts.is_empty() {
        return Vec::new();
    }
    let bnd_edges: Vec<(usize, usize)> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    let mut bnd_adj: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for &(a, b) in &bnd_edges {
        bnd_adj.entry(a).or_default().push(b);
        bnd_adj.entry(b).or_default().push(a);
    }

    let start = *bnd_verts.iter().next().unwrap();
    let mut loop_v = vec![start];
    let mut visited = std::collections::HashSet::new();
    visited.insert(start);
    let mut current = start;
    loop {
        let next = bnd_adj
            .get(&current)
            .and_then(|nbs| nbs.iter().find(|&&n| !visited.contains(&n)).cloned());
        match next {
            Some(n) => {
                loop_v.push(n);
                visited.insert(n);
                current = n;
            }
            None => break,
        }
    }
    loop_v
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
    fn simple_unwrap() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..3 {
            for x in 0..3 {
                let bl = y * 4 + x;
                tris.push([bl, bl + 1, bl + 5]);
                tris.push([bl, bl + 5, bl + 4]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = unwrap_to_uv_atlas(&mesh);
        assert!(result.point_data().tcoords().is_some());
        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_tuples(), mesh.points.len());
    }

    #[test]
    fn two_components() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = unwrap_to_uv_atlas(&mesh);
        assert!(result.point_data().tcoords().is_some());
    }

    #[test]
    fn empty() {
        let result = unwrap_to_uv_atlas(&PolyData::new());
        assert_eq!(result.points.len(), 0);
    }
}
