//! Curvature at multiple scales (multi-scale curvature analysis).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn multi_scale_curvature(mesh: &PolyData, scales: &[f64]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || scales.is_empty() {
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
        let iters = (scale * 10.0).ceil() as usize;
        let dt = scale / iters.max(1) as f64;
        let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
        for _ in 0..iters {
            let mut new_pos = pos.clone();
            for i in 0..n {
                if nb[i].is_empty() {
                    continue;
                }
                let k = nb[i].len() as f64;
                let mut avg = [0.0, 0.0, 0.0];
                for &j in &nb[i] {
                    avg[0] += pos[j][0];
                    avg[1] += pos[j][1];
                    avg[2] += pos[j][2];
                }
                new_pos[i][0] = pos[i][0] + dt * (avg[0] / k - pos[i][0]);
                new_pos[i][1] = pos[i][1] + dt * (avg[1] / k - pos[i][1]);
                new_pos[i][2] = pos[i][2] + dt * (avg[2] / k - pos[i][2]);
            }
            pos = new_pos;
        }
        let curv: Vec<f64> = (0..n)
            .map(|i| {
                let p = mesh.points.get(i);
                let s = pos[i];
                ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2)).sqrt()
            })
            .collect();
        let name = format!("Curvature_s{}", si);
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, curv, 1)));
    }
    result
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
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = multi_scale_curvature(&m, &[0.1, 0.5, 1.0]);
        assert!(r.point_data().get_array("Curvature_s0").is_some());
        assert!(r.point_data().get_array("Curvature_s2").is_some());
    }
}
