//! Compute heat kernel distance between all pairs (approximated via diffusion).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn heat_kernel_distance_from(
    mesh: &PolyData,
    source: usize,
    times: &[f64],
    iterations_per_time: usize,
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
    for (ti, &time) in times.iter().enumerate() {
        let dt = time / iterations_per_time.max(1) as f64;
        let mut heat = vec![0.0f64; n];
        heat[source] = 1.0;
        for _ in 0..iterations_per_time {
            let prev = heat.clone();
            for i in 0..n {
                if nb[i].is_empty() {
                    continue;
                }
                let k = nb[i].len() as f64;
                heat[i] = prev[i] + dt * nb[i].iter().map(|&j| prev[j] - prev[i]).sum::<f64>() / k;
            }
        }
        let name = format!("HKD_t{}", ti);
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, heat, 1)));
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
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = heat_kernel_distance_from(&m, 0, &[0.1, 1.0], 20);
        assert!(r.point_data().get_array("HKD_t0").is_some());
        assert!(r.point_data().get_array("HKD_t1").is_some());
    }
}
