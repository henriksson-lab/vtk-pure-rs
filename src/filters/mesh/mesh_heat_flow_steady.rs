//! Steady-state heat flow with Dirichlet boundary conditions.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn steady_heat(
    mesh: &PolyData,
    hot: &[(usize, f64)],
    cold: &[(usize, f64)],
    iterations: usize,
) -> PolyData {
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
    let mut fixed: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    for &(i, v) in hot {
        fixed.insert(i, v);
    }
    for &(i, v) in cold {
        fixed.insert(i, v);
    }
    let mut t = vec![0.0f64; n];
    for (&i, &v) in &fixed {
        if i < n {
            t[i] = v;
        }
    }
    for _ in 0..iterations {
        let prev = t.clone();
        for i in 0..n {
            if fixed.contains_key(&i) || nb[i].is_empty() {
                continue;
            }
            t[i] = nb[i].iter().map(|&j| prev[j]).sum::<f64>() / nb[i].len() as f64;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Temperature", t, 1)));
    r.point_data_mut().set_active_scalars("Temperature");
    r
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
        let r = steady_heat(&m, &[(0, 100.0)], &[(3, 0.0)], 50);
        let arr = r.point_data().get_array("Temperature").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 100.0).abs() < 1e-5);
        arr.tuple_as_f64(3, &mut buf);
        assert!(buf[0].abs() < 1e-5);
    }
}
