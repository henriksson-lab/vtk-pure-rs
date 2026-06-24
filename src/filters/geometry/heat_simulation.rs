//! Heat equation simulation with sources and boundary conditions.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Simulate heat diffusion with fixed boundary conditions and optional source terms.
pub fn heat_with_sources(
    mesh: &PolyData,
    fixed: &[(usize, f64)],
    source: Option<&str>,
    diffusivity: f64,
    dt: f64,
    steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let bc: std::collections::HashMap<usize, f64> = fixed.iter().cloned().collect();
    let mut temp = vec![0.0f64; n];
    for (&k, &v) in &bc {
        if k < n {
            temp[k] = v;
        }
    }
    let src = source
        .and_then(|name| mesh.point_data().get_array(name))
        .map(|a| {
            let mut buf = [0.0f64];
            (0..n.min(a.num_tuples()))
                .map(|i| {
                    a.tuple_as_f64(i, &mut buf);
                    buf[0]
                })
                .collect::<Vec<f64>>()
        });
    for _ in 0..steps {
        let mut new_t = temp.clone();
        for i in 0..n {
            if bc.contains_key(&i) {
                continue;
            }
            if adj[i].is_empty() {
                continue;
            }
            let lap: f64 =
                adj[i].iter().map(|&j| temp[j] - temp[i]).sum::<f64>() / adj[i].len() as f64;
            let s = src
                .as_ref()
                .map(|sv| if i < sv.len() { sv[i] } else { 0.0 })
                .unwrap_or(0.0);
            new_t[i] = temp[i] + dt * (diffusivity * lap + s);
        }
        temp = new_t;
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Temperature",
            temp,
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
    fn basic() {
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
        let result = heat_with_sources(&mesh, &[(0, 0.0), (24, 100.0)], None, 1.0, 0.1, 100);
        let arr = result.point_data().get_array("Temperature").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] > 10.0 && buf[0] < 90.0);
    }
}
