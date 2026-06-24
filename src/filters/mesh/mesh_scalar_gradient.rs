//! Estimate gradient magnitude of a scalar field on mesh vertices.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_gradient(mesh: &PolyData, scalar_name: &str) -> PolyData {
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
    let grad_mag: Vec<f64> = (0..n)
        .map(|i| {
            if adj[i].is_empty() {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let mut max_grad = 0.0f64;
            for &j in &adj[i] {
                let q = mesh.points.get(j);
                let dist =
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
                if dist > 1e-15 {
                    let grad = (vals[j] - vals[i]).abs() / dist;
                    if grad > max_grad {
                        max_grad = grad;
                    }
                }
            }
            max_grad
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            grad_mag,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("GradientMagnitude");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gradient() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Linear scalar: gradient should be ~1
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = scalar_gradient(&mesh, "f");
        let arr = r.point_data().get_array("GradientMagnitude").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] > 0.5);
    }
}
