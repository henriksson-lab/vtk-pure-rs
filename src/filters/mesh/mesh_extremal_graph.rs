//! Extremal graph: connect critical points of scalar field on mesh.
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn extremal_graph(mesh: &PolyData, array_name: &str) -> PolyData {
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
    // Find extrema (local min/max)
    let mut extrema = Vec::new();
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let is_max = nb[i].iter().all(|&j| vals[j] <= vals[i]);
        let is_min = nb[i].iter().all(|&j| vals[j] >= vals[i]);
        if is_max || is_min {
            extrema.push(i);
        }
    }
    // Connect extrema that share a monotone path
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for i in 0..extrema.len() {
        for j in i + 1..extrema.len() {
            let a = extrema[i];
            let b = extrema[j];
            // Check if directly connected through neighbors
            let connected = nb[a].contains(&b) || (nb[a].iter().any(|&na| nb[na].contains(&b)));
            if connected {
                let ia = *pm.entry(a).or_insert_with(|| {
                    let idx = pts.len();
                    pts.push(mesh.points.get(a));
                    idx
                });
                let ib = *pm.entry(b).or_insert_with(|| {
                    let idx = pts.len();
                    pts.push(mesh.points.get(b));
                    idx
                });
                lines.push_cell(&[ia as i64, ib as i64]);
            }
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
    fn test() {
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
                vec![0.0, 0.5, 0.0, 0.5, 1.0],
                1,
            )));
        let r = extremal_graph(&m, "h");
        assert!(r.points.len() >= 1);
    }
}
