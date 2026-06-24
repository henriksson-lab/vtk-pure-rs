//! Compute aspect ratio for each face.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_aspect_ratio(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() != 3 {
                return 1.0;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let ab = edge_l(a, b);
            let bc = edge_l(b, c);
            let ca = edge_l(c, a);
            let s = (ab + bc + ca) * 0.5;
            let area_sq = s * (s - ab) * (s - bc) * (s - ca);
            if area_sq <= 0.0 {
                return f64::INFINITY;
            }
            let area = area_sq.sqrt();
            let circumr = ab * bc * ca / (4.0 * area);
            let inr = area / s;
            if inr < 1e-15 {
                f64::INFINITY
            } else {
                circumr / (2.0 * inr)
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AspectRatio",
            data,
            1,
        )));
    r
}
pub fn face_skewness(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() != 3 {
                return 0.0;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let angles = [angle_at(a, b, c), angle_at(b, c, a), angle_at(c, a, b)];
            let ideal = 60.0;
            let max_dev = angles
                .iter()
                .map(|&a| (a - ideal).abs())
                .fold(0.0f64, f64::max);
            max_dev / 120.0 // normalized to [0,1]
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Skewness", data, 1)));
    r
}
fn edge_l(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn angle_at(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let v1 = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
    let v2 = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
    let d = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
    let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
    let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
    if l1 > 1e-15 && l2 > 1e-15 {
        (d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees()
    } else {
        0.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equilateral() {
        let h = 3.0f64.sqrt() / 2.0;
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_aspect_ratio(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("AspectRatio")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.05);
    }
    #[test]
    fn test_skewness() {
        let h = 3.0f64.sqrt() / 2.0;
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_skewness(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Skewness")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.1);
    }
}
