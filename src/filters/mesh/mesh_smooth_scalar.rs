//! Smooth a scalar field on mesh vertices using Laplacian averaging.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn smooth_scalar(
    mesh: &PolyData,
    scalar_name: &str,
    iterations: usize,
    lambda: f64,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
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
    let mut vals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals[i] = buf[0];
    }
    for _ in 0..iterations {
        let mut next = vals.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let avg: f64 = adj[i].iter().map(|&j| vals[j]).sum::<f64>() / adj[i].len() as f64;
            next[i] += lambda * (avg - vals[i]);
        }
        vals = next;
    }
    let out_name = format!("{}_smooth", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out_name, vals, 1)));
    result.point_data_mut().set_active_scalars(&out_name);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_smooth_scalar() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 100.0, 0.0, 0.0],
                1,
            )));
        let r = smooth_scalar(&mesh, "v", 5, 0.5);
        let arr = r.point_data().get_array("v_smooth").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b);
        assert!(b[0] < 100.0); // smoothed toward neighbors
    }
}
