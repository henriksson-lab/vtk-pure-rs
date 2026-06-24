//! Multi-scale geodesic heat signature for shape analysis.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn multi_scale_heat_signature(
    mesh: &PolyData,
    source: usize,
    scales: &[f64],
    steps_per_scale: usize,
) -> PolyData {
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
    let mut result = mesh.clone();
    for (si, &scale) in scales.iter().enumerate() {
        let dt = scale / steps_per_scale.max(1) as f64;
        let mut heat = vec![0.0f64; n];
        heat[source] = 1.0;
        for _ in 0..steps_per_scale {
            let prev = heat.clone();
            for i in 0..n {
                if nb[i].is_empty() {
                    continue;
                }
                let k = nb[i].len() as f64;
                heat[i] = prev[i] + dt * nb[i].iter().map(|&j| prev[j] - prev[i]).sum::<f64>() / k;
            }
        }
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &format!("HeatSig_{}", si),
                heat,
                1,
            )));
    }
    result
}
pub fn auto_diffusion_signature(mesh: &PolyData, scales: &[f64], steps: usize) -> PolyData {
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
    let mut result = mesh.clone();
    for (si, &scale) in scales.iter().enumerate() {
        let dt = scale / steps.max(1) as f64;
        let mut diag = vec![0.0f64; n];
        for src in 0..n {
            let mut heat = vec![0.0f64; n];
            heat[src] = 1.0;
            for _ in 0..steps {
                let prev = heat.clone();
                for i in 0..n {
                    if nb[i].is_empty() {
                        continue;
                    }
                    let k = nb[i].len() as f64;
                    heat[i] =
                        prev[i] + dt * nb[i].iter().map(|&j| prev[j] - prev[i]).sum::<f64>() / k;
                }
            }
            diag[src] = heat[src];
        }
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &format!("AutoDiff_{}", si),
                diag,
                1,
            )));
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_heat_sig() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = multi_scale_heat_signature(&m, 0, &[0.1, 1.0], 10);
        assert!(r.point_data().get_array("HeatSig_0").is_some());
        assert!(r.point_data().get_array("HeatSig_1").is_some());
    }
    #[test]
    fn test_auto() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = auto_diffusion_signature(&m, &[0.5], 5);
        assert!(r.point_data().get_array("AutoDiff_0").is_some());
    }
}
