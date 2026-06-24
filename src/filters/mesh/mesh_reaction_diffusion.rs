//! Reaction-diffusion (Gray-Scott) simulation on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn gray_scott(
    mesh: &PolyData,
    steps: usize,
    dt: f64,
    du: f64,
    dv: f64,
    f_rate: f64,
    k_rate: f64,
    seed: usize,
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
    let mut u = vec![1.0f64; n];
    let mut v = vec![0.0f64; n];
    // Seed region
    if seed < n {
        v[seed] = 1.0;
        u[seed] = 0.5;
        for &j in &nb[seed] {
            v[j] = 0.5;
            u[j] = 0.75;
        }
    }
    for _ in 0..steps {
        let mut new_u = u.clone();
        let mut new_v = v.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let lap_u: f64 = nb[i].iter().map(|&j| u[j] - u[i]).sum::<f64>() / k;
            let lap_v: f64 = nb[i].iter().map(|&j| v[j] - v[i]).sum::<f64>() / k;
            let uvv = u[i] * v[i] * v[i];
            new_u[i] = u[i] + dt * (du * lap_u - uvv + f_rate * (1.0 - u[i]));
            new_v[i] = v[i] + dt * (dv * lap_v + uvv - (f_rate + k_rate) * v[i]);
            new_u[i] = new_u[i].clamp(0.0, 1.0);
            new_v[i] = new_v[i].clamp(0.0, 1.0);
        }
        u = new_u;
        v = new_v;
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("U", u, 1)));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("V", v, 1)));
    r.point_data_mut().set_active_scalars("V");
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
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = gray_scott(&m, 50, 0.01, 0.2, 0.1, 0.04, 0.06, 0);
        assert!(r.point_data().get_array("U").is_some());
        assert!(r.point_data().get_array("V").is_some());
    }
}
