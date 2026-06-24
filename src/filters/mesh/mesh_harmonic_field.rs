//! Compute harmonic field (Laplace equation solution with boundary conditions).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn harmonic_field(mesh: &PolyData, fixed: &[(usize, f64)], iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
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
    let mut vals = vec![0.0f64; n];
    let is_fixed: std::collections::HashMap<usize, f64> = fixed.iter().cloned().collect();
    for (&i, &v) in &is_fixed {
        if i < n {
            vals[i] = v;
        }
    }
    for _ in 0..iterations {
        let prev = vals.clone();
        for i in 0..n {
            if is_fixed.contains_key(&i) || nb[i].is_empty() {
                continue;
            }
            vals[i] = nb[i].iter().map(|&j| prev[j]).sum::<f64>() / nb[i].len() as f64;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Harmonic", vals, 1)));
    r.point_data_mut().set_active_scalars("Harmonic");
    r
}
pub fn harmonic_between_vertices(
    mesh: &PolyData,
    v0: usize,
    v1: usize,
    iterations: usize,
) -> PolyData {
    harmonic_field(mesh, &[(v0, 0.0), (v1, 1.0)], iterations)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = harmonic_between_vertices(&m, 0, 3, 50);
        let arr = r.point_data().get_array("Harmonic").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0]).abs() < 1e-5);
        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-5);
    }
}
