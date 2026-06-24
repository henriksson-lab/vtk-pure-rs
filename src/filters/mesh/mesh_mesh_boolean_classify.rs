//! Classify mesh vertices and faces relative to another mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn classify_vertices(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            if point_inside([p[0] + 1e-7, p[1] + 1.3e-7, p[2] + 0.9e-7], reference) {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("InsideRef", data, 1)));
    r
}
pub fn vertex_distance_to_mesh(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let rn = reference.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let mut best = f64::INFINITY;
            for j in 0..rn {
                let q = reference.points.get(j);
                let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
                best = best.min(d);
            }
            best.sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DistToRef", data, 1)));
    r.point_data_mut().set_active_scalars("DistToRef");
    r
}
fn point_inside(p: [f64; 3], mesh: &PolyData) -> bool {
    let mut crossings = 0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let h = [
                0.0 * e2[2] - 0.0 * e2[1],
                0.0 * e2[0] - 1.0 * e2[2],
                1.0 * e2[1] - 0.0 * e2[0],
            ]; // dir=[1,0,0]
            let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
            if det.abs() < 1e-12 {
                continue;
            }
            let inv = 1.0 / det;
            let s = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
            if u < 0.0 || u > 1.0 {
                continue;
            }
            let q = [
                s[1] * e1[2] - s[2] * e1[1],
                s[2] * e1[0] - s[0] * e1[2],
                s[0] * e1[1] - s[1] * e1[0],
            ];
            let v = inv * (1.0 * q[0] + 0.0 * q[1] + 0.0 * q[2]);
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
            if t > 1e-12 {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_classify() {
        let inner = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.05, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let outer = PolyData::from_triangles(
            vec![
                [-5.0, -5.0, -5.0],
                [5.0, -5.0, -5.0],
                [0.0, 5.0, -5.0],
                [0.0, 0.0, 5.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = classify_vertices(&inner, &outer);
        assert!(r.point_data().get_array("InsideRef").is_some());
    }
    #[test]
    fn test_distance() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = vertex_distance_to_mesh(&a, &b);
        let mut buf = [0.0];
        r.point_data()
            .get_array("DistToRef")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
}
