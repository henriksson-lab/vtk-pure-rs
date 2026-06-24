//! Measure mesh symmetry across planes.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn symmetry_error_x(mesh: &PolyData) -> f64 {
    symmetry_error(mesh, 0)
}
pub fn symmetry_error_y(mesh: &PolyData) -> f64 {
    symmetry_error(mesh, 1)
}
pub fn symmetry_error_z(mesh: &PolyData) -> f64 {
    symmetry_error(mesh, 2)
}
fn symmetry_error(mesh: &PolyData, axis: usize) -> f64 {
    let n = mesh.points.len();
    if n == 0 {
        return 0.0;
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut total = 0.0;
    let mut count = 0;
    for i in 0..n {
        let mut p = pts[i];
        p[axis] = -p[axis];
        let mut best = f64::INFINITY;
        for j in 0..n {
            let d = (p[0] - pts[j][0]).powi(2)
                + (p[1] - pts[j][1]).powi(2)
                + (p[2] - pts[j][2]).powi(2);
            best = best.min(d);
        }
        total += best.sqrt();
        count += 1;
    }
    if count > 0 {
        total / count as f64
    } else {
        0.0
    }
}
pub fn attach_symmetry_deviation(mesh: &PolyData, axis: usize) -> PolyData {
    let n = mesh.points.len();
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let mut p = pts[i];
            p[axis] = -p[axis];
            let mut best = f64::INFINITY;
            for j in 0..n {
                let d = (p[0] - pts[j][0]).powi(2)
                    + (p[1] - pts[j][1]).powi(2)
                    + (p[2] - pts[j][2]).powi(2);
                best = best.min(d);
            }
            best.sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SymmetryDev",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("SymmetryDev");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_symmetric() {
        let m = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let e = symmetry_error_x(&m);
        assert!(e < 1e-10);
    }
    #[test]
    fn test_asymmetric() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let e = symmetry_error_x(&m);
        assert!(e > 0.1);
    }
    #[test]
    fn test_attach() {
        let m = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_symmetry_deviation(&m, 0);
        assert!(r.point_data().get_array("SymmetryDev").is_some());
    }
}
