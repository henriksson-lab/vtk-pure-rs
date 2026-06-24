//! Discrete connection and parallel transport on mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn parallel_transport_angle(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
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
    // Compute connection angle per edge (dihedral angle)
    let fnorms: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0, 0.0, 1.0];
            }
            let a = mesh.points.get(c[0] as usize);
            let b = mesh.points.get(c[1] as usize);
            let cc = mesh.points.get(c[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
            let nm = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
            if l < 1e-15 {
                [0.0, 0.0, 1.0]
            } else {
                [nm[0] / l, nm[1] / l, nm[2] / l]
            }
        })
        .collect();
    // Holonomy defect at each vertex (Gaussian curvature)
    let mut angle_sum = vec![0.0f64; n];
    for c in &cells {
        if c.len() < 3 {
            continue;
        }
        let nc = c.len();
        for i in 0..nc {
            let vi = c[i] as usize;
            let prev = c[(i + nc - 1) % nc] as usize;
            let next = c[(i + 1) % nc] as usize;
            let p = mesh.points.get(vi);
            let a = mesh.points.get(prev);
            let b = mesh.points.get(next);
            let va = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
            let vb = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
            let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
            let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
            if la > 1e-15 && lb > 1e-15 {
                let cos =
                    ((va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]) / (la * lb)).clamp(-1.0, 1.0);
                angle_sum[vi] += cos.acos();
            }
        }
    }
    let holonomy: Vec<f64> = angle_sum
        .iter()
        .map(|&s| 2.0 * std::f64::consts::PI - s)
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Holonomy", holonomy, 1,
        )));
    r.point_data_mut().set_active_scalars("Holonomy");
    r
}
pub fn total_holonomy(mesh: &PolyData) -> f64 {
    let r = parallel_transport_angle(mesh);
    let arr = r.point_data().get_array("Holonomy").unwrap();
    let mut buf = [0.0f64];
    let mut total = 0.0;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        total += buf[0];
    }
    total
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = parallel_transport_angle(&m);
        assert!(r.point_data().get_array("Holonomy").is_some());
    }
    #[test]
    fn test_total() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let h = total_holonomy(&m);
        assert!((h - 4.0 * std::f64::consts::PI).abs() < 1.0);
    } // Gauss-Bonnet for sphere ~ 4pi
}
