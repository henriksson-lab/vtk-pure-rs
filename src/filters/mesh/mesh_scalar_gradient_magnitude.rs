//! Compute gradient magnitude of scalar field using finite differences on mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_gradient_magnitude(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals[i] = buf[0];
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    // Least-squares gradient at each vertex
    let grad: Vec<f64> = (0..n)
        .map(|i| {
            if adj[i].len() < 2 {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let mut gx = 0.0f64;
            let mut gy = 0.0f64;
            let mut gz = 0.0f64;
            let mut w_total = 0.0f64;
            for &j in &adj[i] {
                let q = mesh.points.get(j);
                let dx = q[0] - p[0];
                let dy = q[1] - p[1];
                let dz = q[2] - p[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-15);
                let dv = vals[j] - vals[i];
                let w = 1.0 / dist;
                gx += w * dv * dx / dist;
                gy += w * dv * dy / dist;
                gz += w * dv * dz / dist;
                w_total += w;
            }
            if w_total > 1e-15 {
                gx /= w_total;
                gy /= w_total;
                gz /= w_total;
            }
            (gx * gx + gy * gy + gz * gz).sqrt()
        })
        .collect();
    let out = format!("{}_gradmag", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out, grad, 1)));
    result.point_data_mut().set_active_scalars(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_grad_mag() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = scalar_gradient_magnitude(&mesh, "f");
        assert!(r.point_data().get_array("f_gradmag").is_some());
    }
}
