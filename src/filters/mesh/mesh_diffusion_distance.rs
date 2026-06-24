//! Diffusion distance between vertices (multi-scale distance metric).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn diffusion_distance_map(mesh: &PolyData, source: usize, time: f64, steps: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || source >= n {
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
    let dt = time / steps.max(1) as f64;
    // Heat kernel from source
    let mut heat = vec![0.0f64; n];
    heat[source] = 1.0;
    for _ in 0..steps {
        let prev = heat.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let lap: f64 = nb[i].iter().map(|&j| prev[j] - prev[i]).sum::<f64>() / k;
            heat[i] = prev[i] + dt * lap;
        }
    }
    // Diffusion distance = -log(heat) (normalized)
    let max_h = heat.iter().cloned().fold(0.0f64, f64::max).max(1e-30);
    let dist: Vec<f64> = heat
        .iter()
        .map(|&h| {
            let norm = h / max_h;
            if norm > 1e-30 {
                -norm.ln()
            } else {
                30.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DiffusionDist",
            dist,
            1,
        )));
    r.point_data_mut().set_active_scalars("DiffusionDist");
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
        let r = diffusion_distance_map(&m, 0, 1.0, 20);
        let arr = r.point_data().get_array("DiffusionDist").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1.0);
    } // source has small distance
}
