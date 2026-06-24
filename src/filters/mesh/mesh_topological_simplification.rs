//! Topological simplification by persistence-guided cancellation.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn simplify_by_persistence(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
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
    // Iteratively cancel persistence pairs below threshold
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let is_max = nb[i].iter().all(|&j| vals[j] <= vals[i]);
            let is_min = nb[i].iter().all(|&j| vals[j] >= vals[i]);
            if is_max {
                // Find nearest saddle
                let nearest_lower = nb[i].iter().min_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                if let Some(&nl) = nearest_lower {
                    let persistence = vals[i] - vals[nl];
                    if persistence < threshold {
                        vals[i] = vals[nl];
                        changed = true;
                    }
                }
            }
            if is_min {
                let nearest_higher = nb[i].iter().max_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                if let Some(&nh) = nearest_higher {
                    let persistence = vals[nh] - vals[i];
                    if persistence < threshold {
                        vals[i] = vals[nh];
                        changed = true;
                    }
                }
            }
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)));
    r
}
pub fn count_critical_points(mesh: &PolyData, array_name: &str) -> (usize, usize, usize) {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return (0, 0, 0),
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
    let mut mins = 0;
    let mut maxs = 0;
    let mut saddles = 0;
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let lower = nb[i].iter().filter(|&&j| vals[j] < vals[i]).count();
        let upper = nb[i].iter().filter(|&&j| vals[j] > vals[i]).count();
        if lower == 0 && upper > 0 {
            mins += 1;
        } else if upper == 0 && lower > 0 {
            maxs += 1;
        } else if lower > 0 && upper > 0 {
            let changes = nb[i]
                .windows(2)
                .filter(|w| (vals[w[0]] > vals[i]) != (vals[w[1]] > vals[i]))
                .count();
            if changes >= 4 {
                saddles += 1;
            }
        }
    }
    (mins, maxs, saddles)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simplify() {
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
        let r = simplify_by_persistence(&m, "h", 0.2);
        assert!(r.point_data().get_array("h").is_some());
    }
    #[test]
    fn test_count() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [1.0, 0.5, 0.0],
            ],
            vec![[0, 1, 4], [1, 3, 4], [3, 2, 4], [2, 0, 4]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let (mins, maxs, _) = count_critical_points(&m, "h");
        assert!(maxs >= 1);
    }
}
