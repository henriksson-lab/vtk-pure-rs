//! Compute harmonic coordinates for mesh parameterization.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn harmonic_coordinates(
    mesh: &PolyData,
    boundary_u: &[(usize, f64)],
    boundary_v: &[(usize, f64)],
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
    let fixed_u: std::collections::HashMap<usize, f64> = boundary_u.iter().cloned().collect();
    let fixed_v: std::collections::HashMap<usize, f64> = boundary_v.iter().cloned().collect();
    let mut u = vec![0.5f64; n];
    let mut v = vec![0.5f64; n];
    for (&i, &val) in &fixed_u {
        if i < n {
            u[i] = val;
        }
    }
    for (&i, &val) in &fixed_v {
        if i < n {
            v[i] = val;
        }
    }
    for _ in 0..iterations {
        let pu = u.clone();
        let pv = v.clone();
        for i in 0..n {
            if fixed_u.contains_key(&i) {
                continue;
            }
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            u[i] = nb[i].iter().map(|&j| pu[j]).sum::<f64>() / k;
        }
        for i in 0..n {
            if fixed_v.contains_key(&i) {
                continue;
            }
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            v[i] = nb[i].iter().map(|&j| pv[j]).sum::<f64>() / k;
        }
    }
    let data: Vec<f64> = (0..n).flat_map(|i| vec![u[i], v[i]]).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", data, 2)));
    r.point_data_mut().set_active_tcoords("UV");
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
        let r = harmonic_coordinates(&m, &[(0, 0.0), (1, 1.0)], &[(0, 0.0), (2, 1.0)], 50);
        assert!(r.point_data().get_array("UV").is_some());
        assert_eq!(r.point_data().get_array("UV").unwrap().num_components(), 2);
    }
}
