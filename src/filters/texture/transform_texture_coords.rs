//! Transform and animate texture coordinates.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Translate texture coordinates by an offset.
pub fn translate_tcoords(input: &PolyData, du: f64, dv: f64) -> PolyData {
    transform_tcoords_impl(input, |u, v| (u + du, v + dv))
}

/// Scale texture coordinates around (0.5, 0.5).
pub fn scale_tcoords(input: &PolyData, su: f64, sv: f64) -> PolyData {
    transform_tcoords_impl(input, |u, v| (0.5 + (u - 0.5) * su, 0.5 + (v - 0.5) * sv))
}

/// Rotate texture coordinates around (0.5, 0.5) by angle in radians.
pub fn rotate_tcoords(input: &PolyData, angle: f64) -> PolyData {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    transform_tcoords_impl(input, |u, v| {
        let cu = u - 0.5;
        let cv = v - 0.5;
        (0.5 + cu * cos_a - cv * sin_a, 0.5 + cu * sin_a + cv * cos_a)
    })
}

/// Flip texture coordinates (mirror).
pub fn flip_tcoords_u(input: &PolyData) -> PolyData {
    transform_tcoords_impl(input, |u, v| (1.0 - u, v))
}

/// Flip texture coordinates vertically.
pub fn flip_tcoords_v(input: &PolyData) -> PolyData {
    transform_tcoords_impl(input, |u, v| (u, 1.0 - v))
}

/// Tile texture coordinates (repeat).
pub fn tile_tcoords(input: &PolyData, repeat_u: f64, repeat_v: f64) -> PolyData {
    transform_tcoords_impl(input, |u, v| (u * repeat_u, v * repeat_v))
}

/// Clamp texture coordinates to [0, 1].
pub fn clamp_tcoords(input: &PolyData) -> PolyData {
    transform_tcoords_impl(input, |u, v| (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)))
}

/// Wrap texture coordinates to [0, 1) using modulo.
pub fn wrap_tcoords(input: &PolyData) -> PolyData {
    transform_tcoords_impl(input, |u, v| (u.rem_euclid(1.0), v.rem_euclid(1.0)))
}

fn transform_tcoords_impl(input: &PolyData, f: impl Fn(f64, f64) -> (f64, f64)) -> PolyData {
    let tcoords = match input.point_data().tcoords() {
        Some(tc) if tc.num_components() == 2 => tc,
        _ => return input.clone(),
    };

    let n = tcoords.num_tuples();
    let mut new_data = Vec::with_capacity(n * 2);
    let mut buf = [0.0f64; 2];

    for i in 0..n {
        tcoords.tuple_as_f64(i, &mut buf);
        let (nu, nv) = f(buf[0], buf[1]);
        new_data.push(nu);
        new_data.push(nv);
    }

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", new_data, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh_with_tcoords() -> PolyData {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                2,
            )));
        mesh.point_data_mut().set_active_tcoords("TCoords");
        mesh
    }

    #[test]
    fn translate() {
        let mesh = make_mesh_with_tcoords();
        let result = translate_tcoords(&mesh, 0.5, 0.25);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);
        assert!((buf[1] - 0.25).abs() < 1e-10);
    }

    #[test]
    fn scale() {
        let mesh = make_mesh_with_tcoords();
        let result = scale_tcoords(&mesh, 2.0, 2.0);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(1, &mut buf);
        // (1.0, 0.0) scaled around (0.5,0.5) by 2x → (1.5, -0.5)
        assert!((buf[0] - 1.5).abs() < 1e-10);
    }

    #[test]
    fn flip_u() {
        let mesh = make_mesh_with_tcoords();
        let result = flip_tcoords_u(&mesh);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10); // 1-1=0
    }

    #[test]
    fn tile() {
        let mesh = make_mesh_with_tcoords();
        let result = tile_tcoords(&mesh, 3.0, 2.0);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn wrap() {
        let mut mesh = make_mesh_with_tcoords();
        // Set out-of-range tcoords
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![-0.5, 1.5, 2.3, -0.2, 0.5, 0.5],
                2,
            )));
        mesh.point_data_mut().set_active_tcoords("TCoords");
        let result = wrap_tcoords(&mesh);
        let tc = result.point_data().tcoords().unwrap();
        let mut buf = [0.0f64; 2];
        for i in 0..tc.num_tuples() {
            tc.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0 && buf[0] < 1.0, "u={}", buf[0]);
            assert!(buf[1] >= 0.0 && buf[1] < 1.0, "v={}", buf[1]);
        }
    }

    #[test]
    fn no_tcoords() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = translate_tcoords(&mesh, 1.0, 1.0);
        assert_eq!(result.points.len(), 3); // unchanged
    }
}
