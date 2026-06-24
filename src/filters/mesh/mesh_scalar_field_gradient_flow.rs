//! Trace gradient flow lines on a scalar field defined on mesh.
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn gradient_flow_lines(
    mesh: &PolyData,
    array_name: &str,
    seeds: &[usize],
    max_steps: usize,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
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
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for &seed in seeds {
        if seed >= n {
            continue;
        }
        let mut path = vec![seed];
        let mut cur = seed;
        for _ in 0..max_steps {
            let next = nb[cur]
                .iter()
                .filter(|&&j| vals[j] > vals[cur])
                .max_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            match next {
                Some(&nxt) => {
                    if path.contains(&nxt) {
                        break;
                    }
                    path.push(nxt);
                    cur = nxt;
                }
                None => {
                    break;
                }
            }
        }
        if path.len() >= 2 {
            let ids: Vec<i64> = path
                .iter()
                .map(|&v| {
                    let idx = pts.len();
                    pts.push(mesh.points.get(v));
                    idx as i64
                })
                .collect();
            lines.push_cell(&ids);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn gradient_descent_lines(
    mesh: &PolyData,
    array_name: &str,
    seeds: &[usize],
    max_steps: usize,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
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
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for &seed in seeds {
        if seed >= n {
            continue;
        }
        let mut path = vec![seed];
        let mut cur = seed;
        for _ in 0..max_steps {
            let next = nb[cur]
                .iter()
                .filter(|&&j| vals[j] < vals[cur])
                .min_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            match next {
                Some(&nxt) => {
                    if path.contains(&nxt) {
                        break;
                    }
                    path.push(nxt);
                    cur = nxt;
                }
                None => {
                    break;
                }
            }
        }
        if path.len() >= 2 {
            let ids: Vec<i64> = path
                .iter()
                .map(|&v| {
                    let idx = pts.len();
                    pts.push(mesh.points.get(v));
                    idx as i64
                })
                .collect();
            lines.push_cell(&ids);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ascent() {
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
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        let r = gradient_flow_lines(&m, "h", &[0], 10);
        assert!(r.lines.num_cells() >= 1);
    }
    #[test]
    fn test_descent() {
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
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        let r = gradient_descent_lines(&m, "h", &[3], 10);
        assert!(r.lines.num_cells() >= 1);
    }
}
