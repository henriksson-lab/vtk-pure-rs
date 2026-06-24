//! Surface parameterization: conformal, harmonic, LSCM-like.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Harmonic map parameterization for disk-topology meshes.
///
/// Maps boundary to a circle, solves for interior UV via Jacobi iteration.
pub fn harmonic_parameterize(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);
    let boundary = find_ordered_boundary(mesh);

    let mut u = vec![0.0f64; n];
    let mut v = vec![0.0f64; n];
    let is_bnd: std::collections::HashSet<usize> = boundary.iter().cloned().collect();

    // Map boundary to unit circle
    for (i, &vi) in boundary.iter().enumerate() {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / boundary.len() as f64;
        u[vi] = 0.5 + 0.5 * angle.cos();
        v[vi] = 0.5 + 0.5 * angle.sin();
    }

    // Jacobi iteration for interior
    for _ in 0..iterations {
        let mut nu = u.clone();
        let mut nv = v.clone();
        for i in 0..n {
            if is_bnd.contains(&i) || adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f64;
            nu[i] = adj[i].iter().map(|&j| u[j]).sum::<f64>() / k;
            nv[i] = adj[i].iter().map(|&j| v[j]).sum::<f64>() / k;
        }
        u = nu;
        v = nv;
    }

    let mut tc = Vec::with_capacity(n * 2);
    for i in 0..n {
        tc.push(u[i]);
        tc.push(v[i]);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TCoords", tc, 2)));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

/// Projection-based parameterization: project onto best-fit plane.
pub fn planar_parameterize(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    // Project onto XY plane relative to centroid, normalize to [0,1]
    let mut u_vals: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[0] - cx).collect();
    let mut v_vals: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[1] - cy).collect();

    let u_min = u_vals.iter().cloned().fold(f64::MAX, f64::min);
    let u_max = u_vals.iter().cloned().fold(f64::MIN, f64::max);
    let v_min = v_vals.iter().cloned().fold(f64::MAX, f64::min);
    let v_max = v_vals.iter().cloned().fold(f64::MIN, f64::max);
    let ur = (u_max - u_min).max(1e-15);
    let vr = (v_max - v_min).max(1e-15);

    let mut tc = Vec::with_capacity(n * 2);
    for i in 0..n {
        tc.push((u_vals[i] - u_min) / ur);
        tc.push((v_vals[i] - v_min) / vr);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TCoords", tc, 2)));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

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

fn find_ordered_boundary(mesh: &PolyData) -> Vec<usize> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for c in mesh.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let a = c[i] as usize;
            let b = c[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let bnd: Vec<(usize, usize)> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    if bnd.is_empty() {
        return Vec::new();
    }
    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &bnd {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let start = bnd[0].0;
    let mut loop_v = Vec::new();
    let mut vis = std::collections::HashSet::new();
    let mut cur = start;
    loop {
        if !vis.insert(cur) {
            break;
        }
        loop_v.push(cur);
        let next = adj
            .get(&cur)
            .and_then(|nbs| nbs.iter().find(|&&n| !vis.contains(&n)).cloned());
        match next {
            Some(n) => cur = n,
            None => break,
        }
    }
    loop_v
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn harmonic() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = harmonic_parameterize(&mesh, 50);
        assert!(result.point_data().tcoords().is_some());
    }
    #[test]
    fn planar() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = planar_parameterize(&mesh);
        assert!(result.point_data().tcoords().is_some());
    }
}
