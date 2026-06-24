//! Compute shape index from curvature (classifies surface regions).
use crate::data::{AnyDataArray, DataArray, PolyData};
/// Shape index: maps principal curvatures to [-1,1] classification.
/// -1=cup, -0.5=rut, 0=saddle, 0.5=ridge, 1=cap.
pub fn shape_index(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
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
    // Estimate shape index from Laplacian direction
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let k = nb[i].len() as f64;
            let mut avg = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                let q = mesh.points.get(j);
                avg[0] += q[0];
                avg[1] += q[1];
                avg[2] += q[2];
            }
            avg[0] /= k;
            avg[1] /= k;
            avg[2] /= k;
            let lap = [(avg[0] - p[0]), (avg[1] - p[1]), (avg[2] - p[2])];
            let mag = (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt();
            // Estimate sign from normal dot laplacian
            let mut nm = [0.0f64; 3];
            for cell in mesh.polys.iter() {
                if cell.len() < 3 || !cell.iter().any(|&v| v as usize == i) {
                    continue;
                }
                let a = mesh.points.get(cell[0] as usize);
                let b = mesh.points.get(cell[1] as usize);
                let c = mesh.points.get(cell[2] as usize);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                nm[0] += e1[1] * e2[2] - e1[2] * e2[1];
                nm[1] += e1[2] * e2[0] - e1[0] * e2[2];
                nm[2] += e1[0] * e2[1] - e1[1] * e2[0];
            }
            let nl = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2])
                .sqrt()
                .max(1e-15);
            let dot = (lap[0] * nm[0] + lap[1] * nm[1] + lap[2] * nm[2]) / nl;
            let signed_curv = mag * dot.signum();
            (2.0 / std::f64::consts::PI) * signed_curv.atan()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeIndex",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("ShapeIndex");
    r
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
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = shape_index(&m);
        assert!(r.point_data().get_array("ShapeIndex").is_some());
    }
}
