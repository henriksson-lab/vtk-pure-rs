//! Weighted Laplacian smoothing with per-vertex weights.
use crate::data::PolyData;
pub fn weighted_laplacian_smooth(
    mesh: &PolyData,
    weight_array: &str,
    iterations: usize,
    lambda: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let weights = match mesh.point_data().get_array(weight_array) {
        Some(a) if a.num_components() == 1 => {
            let mut buf = [0.0f64];
            (0..a.num_tuples())
                .map(|i| {
                    a.tuple_as_f64(i, &mut buf);
                    buf[0].clamp(0.0, 1.0)
                })
                .collect::<Vec<f64>>()
        }
        _ => vec![1.0; n],
    };
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
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() || i >= weights.len() {
                continue;
            }
            let w = weights[i] * lambda;
            if w < 1e-15 {
                continue;
            }
            let k = nb[i].len() as f64;
            let mut avg = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                avg[0] += prev[j][0];
                avg[1] += prev[j][1];
                avg[2] += prev[j][2];
            }
            pos[i][0] = prev[i][0] + w * (avg[0] / k - prev[i][0]);
            pos[i][1] = prev[i][1] + w * (avg[1] / k - prev[i][1]);
            pos[i][2] = prev[i][2] + w * (avg[2] / k - prev[i][2]);
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let r = weighted_laplacian_smooth(&m, "w", 5, 0.5);
        // Vertices 0,1,2 should not move (weight=0), vertex 3 should smooth
        let p0 = r.points.get(0);
        assert!((p0[0]).abs() < 1e-10);
    }
}
