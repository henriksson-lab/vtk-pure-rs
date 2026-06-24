//! Normalize existing UV texture coordinates to [0,1] range.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn normalize_uv(mesh: &PolyData, uv_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(uv_name) {
        Some(a) if a.num_components() == 2 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let uvs: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1]]
        })
        .collect();
    let u_mn = uvs.iter().map(|uv| uv[0]).fold(f64::INFINITY, f64::min);
    let u_mx = uvs.iter().map(|uv| uv[0]).fold(f64::NEG_INFINITY, f64::max);
    let v_mn = uvs.iter().map(|uv| uv[1]).fold(f64::INFINITY, f64::min);
    let v_mx = uvs.iter().map(|uv| uv[1]).fold(f64::NEG_INFINITY, f64::max);
    let ur = (u_mx - u_mn).max(1e-15);
    let vr = (v_mx - v_mn).max(1e-15);
    let data: Vec<f64> = uvs
        .iter()
        .flat_map(|uv| vec![(uv[0] - u_mn) / ur, (uv[1] - v_mn) / vr])
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(uv_name, data, 2)));
    r
}
pub fn tile_uv(mesh: &PolyData, uv_name: &str, tile_u: f64, tile_v: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(uv_name) {
        Some(a) if a.num_components() == 2 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let data: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            vec![
                (buf[0] * tile_u).rem_euclid(1.0),
                (buf[1] * tile_v).rem_euclid(1.0),
            ]
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(uv_name, data, 2)));
    r
}
pub fn offset_uv(mesh: &PolyData, uv_name: &str, du: f64, dv: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(uv_name) {
        Some(a) if a.num_components() == 2 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let data: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            vec![buf[0] + du, buf[1] + dv]
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(uv_name, data, 2)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "UV",
                vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
                2,
            )));
        let r = normalize_uv(&m, "UV");
        let arr = r.point_data().get_array("UV").unwrap();
        let mut buf = [0.0; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0]).abs() < 1e-10);
    }
    #[test]
    fn test_tile() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "UV",
                vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0],
                2,
            )));
        let r = tile_uv(&m, "UV", 2.0, 2.0);
        assert!(r.point_data().get_array("UV").is_some());
    }
}
