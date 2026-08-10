//! Measure area distortion relative to a reference mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn area_distortion(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let nc = mesh.polys.num_cells().min(reference.polys.num_cells());
    let mc: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let rc: Vec<Vec<i64>> = reference.polys.iter().map(|c| c.to_vec()).collect();
    let data: Vec<f64> = (0..nc)
        .map(|ci| {
            let a1 = tri_area_v(&mc[ci], mesh);
            let a2 = tri_area_v(&rc[ci], reference);
            if a2 < 1e-30 {
                0.0
            } else {
                (a1 / a2).ln()
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AreaDistortion",
            data,
            1,
        )));
    r
}
pub use crate::filters::mesh::conformal_factor::angle_distortion;
fn tri_area_v(cell: &[i64], mesh: &PolyData) -> f64 {
    if cell.len() < 3 {
        return 0.0;
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
    .sqrt()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_area() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = area_distortion(&m, &m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("AreaDistortion")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0].abs() < 1e-10);
    } // no distortion
}
