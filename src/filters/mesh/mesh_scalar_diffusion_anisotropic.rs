//! Anisotropic diffusion of scalar data guided by mesh geometry.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn anisotropic_scalar_diffuse(
    mesh: &PolyData,
    array_name: &str,
    iterations: usize,
    lambda: f64,
    kappa: f64,
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
    let k2 = kappa * kappa;
    for _ in 0..iterations {
        let prev = vals.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let mut flux = 0.0;
            for &j in &nb[i] {
                let diff = prev[j] - prev[i];
                let g = (-diff * diff / k2).exp(); // Perona-Malik conductance
                flux += g * diff;
            }
            vals[i] = prev[i] + lambda * flux / nb[i].len() as f64;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)));
    r
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
        let r = anisotropic_scalar_diffuse(&m, "s", 10, 0.25, 5.0);
        assert!(r.point_data().get_array("s").is_some());
    }
}
