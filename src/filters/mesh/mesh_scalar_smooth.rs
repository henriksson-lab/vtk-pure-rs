//! Smooth a scalar point data array using Laplacian diffusion.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn smooth_scalar(
    mesh: &PolyData,
    array_name: &str,
    iterations: usize,
    lambda: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    for _ in 0..iterations {
        let prev = vals.clone();
        for i in 0..n {
            if nb[i].is_empty() || i >= prev.len() {
                continue;
            }
            let avg: f64 = nb[i].iter().map(|&j| prev[j]).sum::<f64>() / nb[i].len() as f64;
            vals[i] = prev[i] + lambda * (avg - prev[i]);
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)));
    r
}
pub fn smooth_scalar_preserve_range(
    mesh: &PolyData,
    array_name: &str,
    iterations: usize,
    lambda: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let orig: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = orig.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = orig.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let r = smooth_scalar(mesh, array_name, iterations, lambda);
    let sarr = r.point_data().get_array(array_name).unwrap();
    let svals: Vec<f64> = (0..sarr.num_tuples())
        .map(|i| {
            sarr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let smn = svals.iter().cloned().fold(f64::INFINITY, f64::min);
    let smx = svals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let sr = (smx - smn).max(1e-15);
    let or = mx - mn;
    let data: Vec<f64> = svals.iter().map(|&v| mn + (v - smn) / sr * or).collect();
    let mut result = r;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 10.0, 5.0, 10.0],
                1,
            )));
        let r = smooth_scalar(&m, "s", 3, 0.5);
        assert!(r.point_data().get_array("s").is_some());
    }
    #[test]
    fn test_preserve() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 10.0, 5.0],
                1,
            )));
        let r = smooth_scalar_preserve_range(&m, "s", 5, 0.5);
        let arr = r.point_data().get_array("s").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        let mn = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let mx = buf[0];
        assert!((mn).abs() < 1e-10);
        assert!((mx - 10.0).abs() < 1e-10);
    }
}
