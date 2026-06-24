//! Scalar field operations on meshes: gradient magnitude, Laplacian, divergence.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete gradient magnitude of a scalar field on mesh.
pub fn gradient_magnitude_on_mesh(mesh: &PolyData, array_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let adj = build_adj(mesh, n);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut grad_mag = Vec::with_capacity(n);
    for i in 0..n {
        if adj[i].is_empty() {
            grad_mag.push(0.0);
            continue;
        }
        let pi = mesh.points.get(i);
        let mut gx = 0.0;
        let mut gy = 0.0;
        let mut gz = 0.0;
        let mut w_sum = 0.0;
        for &j in &adj[i] {
            let pj = mesh.points.get(j);
            let dx = pj[0] - pi[0];
            let dy = pj[1] - pi[1];
            let dz = pj[2] - pi[2];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d > 1e-15 {
                let dv = vals[j] - vals[i];
                let w = 1.0 / d;
                gx += w * dv * dx / d;
                gy += w * dv * dy / d;
                gz += w * dv * dz / d;
                w_sum += w;
            }
        }
        if w_sum > 1e-15 {
            grad_mag
                .push(((gx / w_sum).powi(2) + (gy / w_sum).powi(2) + (gz / w_sum).powi(2)).sqrt());
        } else {
            grad_mag.push(0.0);
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            grad_mag,
            1,
        )));
    result
}

/// Compute discrete Laplacian of a scalar field on mesh.
pub fn laplacian_on_mesh(mesh: &PolyData, array_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let adj = build_adj(mesh, n);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut laplacian = Vec::with_capacity(n);
    for i in 0..n {
        if adj[i].is_empty() {
            laplacian.push(0.0);
            continue;
        }
        let lap: f64 = adj[i].iter().map(|&j| vals[j] - vals[i]).sum::<f64>() / adj[i].len() as f64;
        laplacian.push(lap);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Laplacian",
            laplacian,
            1,
        )));
    result
}

/// Diffuse (smooth) a scalar field on mesh.
pub fn diffuse_scalar(
    mesh: &PolyData,
    array_name: &str,
    factor: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let adj = build_adj(mesh, n);
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    for _ in 0..iterations {
        let mut new_vals = vals.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let avg: f64 = adj[i].iter().map(|&j| vals[j]).sum::<f64>() / adj[i].len() as f64;
            new_vals[i] = vals[i] * (1.0 - factor) + avg * factor;
        }
        vals = new_vals;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)));
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
    fn grad_mag() {
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
        let mut mesh = PolyData::from_triangles(pts, tris);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                (0..25).map(|i| (i % 5) as f64).collect(),
                1,
            )));
        let result = gradient_magnitude_on_mesh(&mesh, "f");
        assert!(result.point_data().get_array("GradientMagnitude").is_some());
    }
    #[test]
    fn laplacian() {
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
        let mut mesh = PolyData::from_triangles(pts, tris);
        let vals: Vec<f64> = (0..25)
            .map(|i| {
                let x = (i % 5) as f64 - 2.0;
                let y = (i / 5) as f64 - 2.0;
                x * x + y * y
            })
            .collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vals, 1)));
        let result = laplacian_on_mesh(&mesh, "f");
        assert!(result.point_data().get_array("Laplacian").is_some());
    }
    #[test]
    fn diffuse() {
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
        let mut mesh = PolyData::from_triangles(pts, tris);
        let mut vals = vec![0.0; 25];
        vals[12] = 100.0; // spike at center
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vals, 1)));
        let result = diffuse_scalar(&mesh, "f", 0.5, 5);
        let arr = result.point_data().get_array("f").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] < 100.0); // smoothed
    }
}
