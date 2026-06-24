//! Compute area associated with each vertex (Voronoi area approximation).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn vertex_area(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut areas = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let mut tri_area = 0.0;
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            tri_area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }
        let share = tri_area / cell.len() as f64;
        for &v in cell {
            areas[v as usize] += share;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexArea",
            areas,
            1,
        )));
    r.point_data_mut().set_active_scalars("VertexArea");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = vertex_area(&m);
        let arr = r.point_data().get_array("VertexArea").unwrap();
        let mut buf = [0.0];
        let mut total = 0.0;
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            total += buf[0];
        }
        assert!((total - 2.0).abs() < 1e-10);
    } // total vertex area = triangle area
}
