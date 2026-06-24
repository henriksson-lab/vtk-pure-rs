//! Compute statistics over N-ring neighborhood of each vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn ring_mean(mesh: &PolyData, array_name: &str, rings: usize) -> PolyData {
    ring_stat(mesh, array_name, rings, "RingMean", |vals| {
        vals.iter().sum::<f64>() / vals.len().max(1) as f64
    })
}
pub fn ring_max(mesh: &PolyData, array_name: &str, rings: usize) -> PolyData {
    ring_stat(mesh, array_name, rings, "RingMax", |vals| {
        vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    })
}
pub fn ring_min(mesh: &PolyData, array_name: &str, rings: usize) -> PolyData {
    ring_stat(mesh, array_name, rings, "RingMin", |vals| {
        vals.iter().cloned().fold(f64::INFINITY, f64::min)
    })
}
pub fn ring_range(mesh: &PolyData, array_name: &str, rings: usize) -> PolyData {
    ring_stat(mesh, array_name, rings, "RingRange", |vals| {
        let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        mx - mn
    })
}
fn ring_stat(
    mesh: &PolyData,
    array_name: &str,
    rings: usize,
    out_name: &str,
    f: impl Fn(&[f64]) -> f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
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
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let mut visited = vec![false; n];
            visited[i] = true;
            let mut frontier = vec![i];
            for _ in 0..rings {
                let mut next = Vec::new();
                for &v in &frontier {
                    for &u in &nb[v] {
                        if !visited[u] {
                            visited[u] = true;
                            next.push(u);
                        }
                    }
                }
                frontier = next;
            }
            let ring_vals: Vec<f64> = (0..n)
                .filter(|&j| visited[j] && j < vals.len())
                .map(|j| vals[j])
                .collect();
            if ring_vals.is_empty() {
                0.0
            } else {
                f(&ring_vals)
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(out_name, data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mean() {
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
        let r = ring_mean(&m, "s", 1);
        assert!(r.point_data().get_array("RingMean").is_some());
    }
    #[test]
    fn test_range() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 10.0, 5.0],
                1,
            )));
        let r = ring_range(&m, "s", 1);
        assert!(r.point_data().get_array("RingRange").is_some());
    }
}
