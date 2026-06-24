//! Wave propagation simulation on mesh connectivity.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Simulate wave propagation from source vertices.
///
/// Uses the wave equation: u_tt = c² * Laplacian(u) with damping.
pub fn wave_propagate(
    mesh: &PolyData,
    sources: &[(usize, f64)],
    wave_speed: f64,
    damping: f64,
    dt: f64,
    steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut u = vec![0.0f64; n];
    let mut u_prev = vec![0.0f64; n];
    for &(si, amp) in sources {
        if si < n {
            u[si] = amp;
            u_prev[si] = amp;
        }
    }

    let c2 = wave_speed * wave_speed;
    for _ in 0..steps {
        let mut u_next = vec![0.0f64; n];
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let lap: f64 = adj[i].iter().map(|&j| u[j] - u[i]).sum::<f64>() / adj[i].len() as f64;
            u_next[i] =
                2.0 * u[i] - u_prev[i] + dt * dt * c2 * lap - damping * dt * (u[i] - u_prev[i]);
        }
        u_prev = u;
        u = u_next;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WaveAmplitude",
            u,
            1,
        )));
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wave() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = wave_propagate(&mesh, &[(50, 1.0)], 1.0, 0.01, 0.1, 20);
        let arr = result.point_data().get_array("WaveAmplitude").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(50, &mut buf);
        assert!(buf[0].abs() > 0.0);
    }
}
