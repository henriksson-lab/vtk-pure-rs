//! Discrete Morse gradient field on mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn discrete_morse_gradient(mesh: &PolyData, array_name: &str) -> PolyData {
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
    // Discrete gradient: pair each vertex with steepest ascending edge
    let mut paired_with = vec![usize::MAX; n]; // vertex -> paired neighbor (steepest ascent)
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let steepest = nb[i].iter().max_by(|&&a, &&b| {
            let da = vals[a] - vals[i];
            let db = vals[b] - vals[i];
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(&s) = steepest {
            if vals[s] > vals[i] {
                paired_with[i] = s;
            }
        }
    }
    // Critical cells: unpaired vertices
    let critical: Vec<f64> = (0..n)
        .map(|i| {
            if paired_with[i] == usize::MAX && !nb[i].iter().any(|&j| paired_with[j] == i) {
                if nb[i].iter().all(|&j| vals[j] >= vals[i]) {
                    -1.0
                }
                // minimum
                else if nb[i].iter().all(|&j| vals[j] <= vals[i]) {
                    1.0
                }
                // maximum
                else {
                    0.5
                } // saddle-like
            } else {
                0.0
            } // regular
        })
        .collect();
    // Gradient direction vectors
    let grad: Vec<f64> = (0..n)
        .flat_map(|i| {
            if paired_with[i] < n {
                let p = mesh.points.get(i);
                let q = mesh.points.get(paired_with[i]);
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
                vec![d[0] / l, d[1] / l, d[2] / l]
            } else {
                vec![0.0, 0.0, 0.0]
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorseCritical",
            critical,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorseGradient",
            grad,
            3,
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
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 3.0, 2.0],
                1,
            )));
        let r = discrete_morse_gradient(&m, "h");
        assert!(r.point_data().get_array("MorseCritical").is_some());
        assert!(r.point_data().get_array("MorseGradient").is_some());
        assert_eq!(
            r.point_data()
                .get_array("MorseGradient")
                .unwrap()
                .num_components(),
            3
        );
    }
}
