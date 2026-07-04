//! Compute aspect ratio for each face.
use crate::data::{AnyDataArray, DataArray, PolyData};

const VERDICT_DBL_MAX: f64 = 1.0e30;
const VERDICT_DBL_MIN: f64 = 2.2204460492503131e-15;
const ASPECT_RATIO_NORMAL_COEFF: f64 = 1.7320508075688772 / 6.0;

pub fn face_aspect_ratio(mesh: &PolyData) -> PolyData {
    let num_points = mesh.points.len();
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() != 3 {
                return 0.0;
            }
            let Some(a_id) = valid_point_id(cell[0], num_points) else {
                return 0.0;
            };
            let Some(b_id) = valid_point_id(cell[1], num_points) else {
                return 0.0;
            };
            let Some(c_id) = valid_point_id(cell[2], num_points) else {
                return 0.0;
            };
            let a = mesh.points.get(a_id);
            let b = mesh.points.get(b_id);
            let c = mesh.points.get(c_id);
            let ab = edge_l(a, b);
            let bc = edge_l(b, c);
            let ca = edge_l(c, a);
            let max_edge = ab.max(bc).max(ca);
            let denominator = cross_len(
                [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                [c[0] - b[0], c[1] - b[1], c[2] - b[2]],
            );
            if denominator < VERDICT_DBL_MIN {
                VERDICT_DBL_MAX
            } else {
                (ASPECT_RATIO_NORMAL_COEFF * max_edge * (ab + bc + ca) / denominator)
                    .clamp(-VERDICT_DBL_MAX, VERDICT_DBL_MAX)
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
    let num_points = mesh.points.len();
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() != 3 {
                return 0.0;
            }
            let Some(a_id) = valid_point_id(cell[0], num_points) else {
                return 0.0;
            };
            let Some(b_id) = valid_point_id(cell[1], num_points) else {
                return 0.0;
            };
            let Some(c_id) = valid_point_id(cell[2], num_points) else {
                return 0.0;
            };
            let a = mesh.points.get(a_id);
            let b = mesh.points.get(b_id);
            let c = mesh.points.get(c_id);
            let angles = [angle_at(a, b, c), angle_at(b, c, a), angle_at(c, a, b)];
            let min_angle = angles.iter().copied().fold(360.0f64, f64::min);
            let max_angle = angles.iter().copied().fold(0.0f64, f64::max);
            ((max_angle - 60.0) / 120.0).max((60.0 - min_angle) / 60.0)
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
fn cross_len(a: [f64; 3], b: [f64; 3]) -> f64 {
    let c = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt()
}
fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
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
