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
            (0..n)
                .map(|i| {
                    if i < a.num_tuples() {
                        a.tuple_as_f64(i, &mut buf);
                        buf[0].clamp(0.0, 1.0)
                    } else {
                        1.0
                    }
                })
                .collect::<Vec<f64>>()
        }
        _ => vec![1.0; n],
    };
    let nb = build_neighbors(mesh, n);
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
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
    for (i, p) in pos.iter().enumerate() {
        r.points.set(i, *p);
    }
    r
}

fn build_neighbors(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            add_neighbor_edge(cell[i], cell[(i + 1) % nc], n, &mut nb);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_edge(edge[0], edge[1], n, &mut nb);
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            add_neighbor_edge(tri[0], tri[1], n, &mut nb);
            add_neighbor_edge(tri[1], tri[2], n, &mut nb);
            add_neighbor_edge(tri[2], tri[0], n, &mut nb);
        }
    }
    nb
}

fn add_neighbor_edge(a: i64, b: i64, n: usize, nb: &mut [Vec<usize>]) {
    let (Some(a), Some(b)) = (valid_point_index(a, n), valid_point_index(b, n)) else {
        return;
    };
    if a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
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

    #[test]
    fn short_weight_array_uses_available_weights() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 1.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![0.0, 0.0, 0.0],
                1,
            )));

        let r = weighted_laplacian_smooth(&m, "w", 1, 0.5);

        for i in 0..3 {
            assert_eq!(r.points.get(i), m.points.get(i));
        }
        assert_ne!(r.points.get(3), m.points.get(3));
    }
}
