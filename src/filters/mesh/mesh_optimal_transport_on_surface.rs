//! Optimal transport displacement on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn transport_displacement(
    mesh: &PolyData,
    source_array: &str,
    target_array: &str,
    iterations: usize,
) -> PolyData {
    let sarr = match mesh.point_data().get_array(source_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let tarr = match mesh.point_data().get_array(target_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let src: Vec<f64> = (0..sarr.num_tuples())
        .map(|i| {
            sarr.tuple_as_f64(i, &mut buf);
            buf[0].max(0.0)
        })
        .collect();
    let tgt: Vec<f64> = (0..tarr.num_tuples())
        .map(|i| {
            tarr.tuple_as_f64(i, &mut buf);
            buf[0].max(0.0)
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
    // Iterative transport: move mass from source toward target
    let mut current = src.clone();
    for _ in 0..iterations {
        let prev = current.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let excess = prev[i] - tgt[i];
            if excess.abs() < 1e-10 {
                continue;
            }
            let k = nb[i].len() as f64;
            let share = excess * 0.1 / k;
            current[i] -= excess * 0.1;
            for &j in &nb[i] {
                current[j] += share;
            }
        }
    }
    // Compute transport cost
    let cost: f64 = (0..n).map(|i| (current[i] - src[i]).abs()).sum::<f64>();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Transported",
            current,
            1,
        )));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
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
                "src",
                vec![1.0, 0.0, 0.0, 0.0],
                1,
            )));
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "tgt",
                vec![0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let r = transport_displacement(&m, "src", "tgt", 50);
        assert!(r.point_data().get_array("Transported").is_some());
    }
}
