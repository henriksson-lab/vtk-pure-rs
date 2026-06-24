//! Measure planarity of mesh faces (deviation from best-fit plane).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_planarity(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() <= 3 {
                return 0.0;
            } // triangles are always planar
            let pts: Vec<[f64; 3]> = cell.iter().map(|&v| mesh.points.get(v as usize)).collect();
            let n = pts.len();
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for p in &pts {
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            cx /= n as f64;
            cy /= n as f64;
            cz /= n as f64;
            // Compute normal from first 3 vertices
            let e1 = [
                pts[1][0] - pts[0][0],
                pts[1][1] - pts[0][1],
                pts[1][2] - pts[0][2],
            ];
            let e2 = [
                pts[2][0] - pts[0][0],
                pts[2][1] - pts[0][1],
                pts[2][2] - pts[0][2],
            ];
            let nm = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let nl = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
            if nl < 1e-15 {
                return 0.0;
            }
            let nn = [nm[0] / nl, nm[1] / nl, nm[2] / nl];
            // Max distance from plane
            let mut max_d = 0.0f64;
            for p in &pts {
                let d = ((p[0] - cx) * nn[0] + (p[1] - cy) * nn[1] + (p[2] - cz) * nn[2]).abs();
                max_d = max_d.max(d);
            }
            max_d
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Planarity", data, 1)));
    r
}
pub fn non_planar_face_count(mesh: &PolyData, tolerance: f64) -> usize {
    let r = face_planarity(mesh);
    let arr = r.cell_data().get_array("Planarity").unwrap();
    let mut buf = [0.0f64];
    let mut count = 0;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] > tolerance {
            count += 1;
        }
    }
    count
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tri() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_planarity(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Planarity")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1e-10);
    }
    #[test]
    fn test_quad() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([1.0, 1.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.polys.push_cell(&[0, 1, 2, 3]);
        assert_eq!(non_planar_face_count(&m, 1e-10), 0);
    }
}
