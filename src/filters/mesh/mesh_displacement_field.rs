//! Apply a displacement field from point data to vertex positions.
use crate::data::{Points, PolyData};

pub fn apply_displacement(mesh: &PolyData, array_name: &str, scale: f64) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let mut pts = Points::<f64>::new();
    let mut buf = vec![0.0f64; nc];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let p = mesh.points.get(i);
        let dx = if nc > 0 { buf[0] * scale } else { 0.0 };
        let dy = if nc > 1 { buf[1] * scale } else { 0.0 };
        let dz = if nc > 2 { buf[2] * scale } else { 0.0 };
        pts.push([p[0] + dx, p[1] + dy, p[2] + dz]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_displacement() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "disp",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));
        let r = apply_displacement(&mesh, "disp", 2.0);
        let p = r.points.get(0);
        assert!((p[2] - 2.0).abs() < 1e-9);
    }
}
