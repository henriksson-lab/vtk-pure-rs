//! Compute mean edge length per vertex and store as scalar.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn mean_edge_length(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut sum_len = vec![0.0f64; n];
    let mut count = vec![0u32; n];
    let mut seen = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= n || b >= n {
                continue;
            }
            let e = if a < b { (a, b) } else { (b, a) };
            if !seen.insert(e) {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            sum_len[a] += d;
            count[a] += 1;
            sum_len[b] += d;
            count[b] += 1;
        }
    }
    let avg: Vec<f64> = (0..n)
        .map(|i| {
            if count[i] > 0 {
                sum_len[i] / count[i] as f64
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanEdgeLength",
            avg,
            1,
        )));
    result.point_data_mut().set_active_scalars("MeanEdgeLength");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mean_edge() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mean_edge_length(&mesh);
        let arr = r.point_data().get_array("MeanEdgeLength").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] > 0.5);
    }
}
