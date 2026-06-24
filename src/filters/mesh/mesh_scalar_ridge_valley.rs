//! Detect ridges and valleys on mesh scalar field.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn detect_ridges_valleys(mesh: &PolyData, array_name: &str) -> PolyData {
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
    // Ridge: vertex higher than all neighbors; Valley: lower than all
    // Saddle: has both higher and lower neighbors with sign changes
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let vi = vals[i];
            let higher = nb[i].iter().filter(|&&j| vals[j] > vi).count();
            let lower = nb[i].iter().filter(|&&j| vals[j] < vi).count();
            if higher == 0 && lower > 0 {
                1.0
            }
            // ridge/peak
            else if lower == 0 && higher > 0 {
                -1.0
            }
            // valley/pit
            else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RidgeValley",
            data,
            1,
        )));
    r
}
pub fn extract_ridge_lines(mesh: &PolyData, array_name: &str) -> PolyData {
    let rv = detect_ridges_valleys(mesh, array_name);
    let arr = rv.point_data().get_array("RidgeValley").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut ridge_verts: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] > 0.5 {
            ridge_verts.insert(i);
        }
    }
    let mut pts = crate::data::Points::<f64>::new();
    let mut lines = crate::data::CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if ridge_verts.contains(&a) && ridge_verts.contains(&b) {
                let ia = *pm.entry(a).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(a));
                    i
                });
                let ib = *pm.entry(b).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(b));
                    i
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
    fn test_rv() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let r = detect_ridges_valleys(&m, "h");
        assert!(r.point_data().get_array("RidgeValley").is_some());
    }
}
