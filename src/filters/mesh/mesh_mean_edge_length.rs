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
        for i in 0..cell.len() {
            accumulate_edge(
                mesh,
                n,
                cell[i],
                cell[(i + 1) % cell.len()],
                &mut seen,
                &mut sum_len,
                &mut count,
            );
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            accumulate_edge(
                mesh,
                n,
                edge[0],
                edge[1],
                &mut seen,
                &mut sum_len,
                &mut count,
            );
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            accumulate_edge(mesh, n, tri[0], tri[1], &mut seen, &mut sum_len, &mut count);
            accumulate_edge(mesh, n, tri[1], tri[2], &mut seen, &mut sum_len, &mut count);
            accumulate_edge(mesh, n, tri[2], tri[0], &mut seen, &mut sum_len, &mut count);
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

fn accumulate_edge(
    mesh: &PolyData,
    n: usize,
    a_id: i64,
    b_id: i64,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    sum_len: &mut [f64],
    count: &mut [u32],
) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    let e = if a < b { (a, b) } else { (b, a) };
    if !seen.insert(e) {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    sum_len[a] += d;
    count[a] += 1;
    sum_len[b] += d;
    count[b] += 1;
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
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
