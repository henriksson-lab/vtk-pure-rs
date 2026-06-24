//! Simple dual contouring from scalar field on mesh.
use crate::data::{CellArray, Points, PolyData};
pub fn dual_contour_on_mesh(mesh: &PolyData, array_name: &str, isovalue: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    // For each face, compute dual vertex (centroid if face crosses iso)
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut dual_pts = Points::<f64>::new();
    let mut face_has_dual = vec![false; cells.len()];
    let mut face_dual_idx = vec![0usize; cells.len()];
    for (ci, c) in cells.iter().enumerate() {
        if c.is_empty() {
            continue;
        }
        let signs: Vec<bool> = c.iter().map(|&v| vals[v as usize] >= isovalue).collect();
        let crosses = signs.windows(2).any(|w| w[0] != w[1])
            || (*signs.first().unwrap() != *signs.last().unwrap());
        if crosses {
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &v in c {
                let p = mesh.points.get(v as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = c.len() as f64;
            face_dual_idx[ci] = dual_pts.len();
            dual_pts.push([cx / n, cy / n, cz / n]);
            face_has_dual[ci] = true;
        }
    }
    // Connect dual vertices across shared edges
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let nc = c.len();
        for i in 0..nc {
            let a = c[i] as usize;
            let b = c[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut lines = CellArray::new();
    for (_, faces) in &ef {
        if faces.len() == 2 && face_has_dual[faces[0]] && face_has_dual[faces[1]] {
            lines.push_cell(&[
                face_dual_idx[faces[0]] as i64,
                face_dual_idx[faces[1]] as i64,
            ]);
        }
    }
    let mut r = PolyData::new();
    r.points = dual_pts;
    r.lines = lines;
    r
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
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 2.0, 1.0, 3.0],
                1,
            )));
        let r = dual_contour_on_mesh(&m, "s", 1.5);
        assert!(r.points.len() >= 1);
    }
}
