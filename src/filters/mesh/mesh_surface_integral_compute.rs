//! Compute surface integrals of scalar/vector fields over mesh.
use crate::data::PolyData;
pub fn integrate_scalar(mesh: &PolyData, array_name: &str) -> f64 {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0.0,
    };
    let mut buf = [0.0f64];
    let mut total = 0.0;
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
            let area = 0.5
                * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                    + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                    + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                .sqrt();
            // Average scalar over triangle vertices
            arr.tuple_as_f64(cell[0] as usize, &mut buf);
            let v0 = buf[0];
            arr.tuple_as_f64(cell[i] as usize, &mut buf);
            let v1 = buf[0];
            arr.tuple_as_f64(cell[i + 1] as usize, &mut buf);
            let v2 = buf[0];
            total += area * (v0 + v1 + v2) / 3.0;
        }
    }
    total
}
pub fn integrate_constant(mesh: &PolyData) -> f64 {
    // Surface area
    let mut total = 0.0;
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
            total += 0.5
                * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                    + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                    + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                .sqrt();
        }
    }
    total
}
pub fn weighted_centroid(mesh: &PolyData, array_name: &str) -> [f64; 3] {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return [0.0; 3],
    };
    let mut buf = [0.0f64];
    let mut wsum = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    let n = mesh.points.len();
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let w = buf[0].abs();
        let p = mesh.points.get(i);
        cx += p[0] * w;
        cy += p[1] * w;
        cz += p[2] * w;
        wsum += w;
    }
    if wsum > 1e-30 {
        [cx / wsum, cy / wsum, cz / wsum]
    } else {
        [0.0; 3]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_area() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let area = integrate_constant(&m);
        assert!((area - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_scalar() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 1.0, 1.0],
                1,
            )));
        let integral = integrate_scalar(&m, "s");
        assert!((integral - 2.0).abs() < 1e-10);
    } // area * 1.0
    #[test]
    fn test_centroid() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![1.0, 1.0, 1.0],
                1,
            )));
        let c = weighted_centroid(&m, "w");
        assert!((c[0] - 2.0 / 3.0).abs() < 1e-5);
    }
}
