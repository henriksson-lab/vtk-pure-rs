//! Cancel persistence pairs in Morse-Smale complex.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn cancel_pairs(
    mesh: &PolyData,
    array_name: &str,
    max_cancellations: usize,
    persistence_threshold: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..arr.num_tuples())
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
    for _ in 0..max_cancellations {
        // Find minimum persistence critical pair
        let mut best_pair: (usize, usize, f64) = (0, 0, f64::INFINITY);
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let is_max = nb[i].iter().all(|&j| vals[j] <= vals[i]);
            let is_min = nb[i].iter().all(|&j| vals[j] >= vals[i]);
            if is_max {
                for &j in &nb[i] {
                    let p = vals[i] - vals[j];
                    if p < best_pair.2 {
                        best_pair = (i, j, p);
                    }
                }
            }
            if is_min {
                for &j in &nb[i] {
                    let p = vals[j] - vals[i];
                    if p < best_pair.2 {
                        best_pair = (j, i, p);
                    }
                }
            }
        }
        if best_pair.2 >= persistence_threshold || best_pair.2 >= f64::INFINITY {
            break;
        }
        let avg = (vals[best_pair.0] + vals[best_pair.1]) / 2.0;
        vals[best_pair.0] = avg;
        vals[best_pair.1] = avg;
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)));
    r
}
pub fn persistence_count(mesh: &PolyData, array_name: &str) -> usize {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0,
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
    let mut count = 0;
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let is_max = nb[i].iter().all(|&j| vals[j] <= vals[i]);
        let is_min = nb[i].iter().all(|&j| vals[j] >= vals[i]);
        if is_max || is_min {
            count += 1;
        }
    }
    count
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cancel() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 0.1, 1.0, 0.05],
                1,
            )));
        let before = persistence_count(&m, "h");
        let r = cancel_pairs(&m, "h", 10, 0.2);
        let after = persistence_count(&r, "h");
        assert!(after <= before);
    }
    #[test]
    fn test_count() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let c = persistence_count(&m, "h");
        assert!(c >= 1);
    }
}
