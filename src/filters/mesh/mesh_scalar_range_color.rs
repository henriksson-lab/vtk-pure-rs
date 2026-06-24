//! Map scalar range to custom color gradient.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn scalar_to_rainbow(mesh: &PolyData, array_name: &str) -> PolyData {
    scalar_to_gradient(
        mesh,
        array_name,
        &[
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    )
}
pub fn scalar_to_viridis(mesh: &PolyData, array_name: &str) -> PolyData {
    scalar_to_gradient(
        mesh,
        array_name,
        &[
            [0.267, 0.004, 0.329],
            [0.282, 0.140, 0.458],
            [0.127, 0.566, 0.551],
            [0.544, 0.773, 0.247],
            [0.993, 0.906, 0.144],
        ],
    )
}
pub fn scalar_to_hot(mesh: &PolyData, array_name: &str) -> PolyData {
    scalar_to_gradient(
        mesh,
        array_name,
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ],
    )
}
pub fn scalar_to_cool_warm(mesh: &PolyData, array_name: &str) -> PolyData {
    scalar_to_gradient(
        mesh,
        array_name,
        &[
            [0.231, 0.298, 0.753],
            [0.865, 0.865, 0.865],
            [0.706, 0.016, 0.150],
        ],
    )
}
pub fn scalar_to_gradient(mesh: &PolyData, array_name: &str, gradient: &[[f64; 3]]) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let ng = gradient.len();
    if ng < 2 {
        return mesh.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let colors: Vec<f64> = vals
        .iter()
        .flat_map(|&v| {
            let t = (v - mn) / range * (ng - 1) as f64;
            let i0 = (t.floor() as usize).min(ng - 2);
            let frac = t - i0 as f64;
            let c0 = gradient[i0];
            let c1 = gradient[i0 + 1];
            vec![
                (c0[0] * (1.0 - frac) + c1[0] * frac) * 255.0,
                (c0[1] * (1.0 - frac) + c1[1] * frac) * 255.0,
                (c0[2] * (1.0 - frac) + c1[2] * frac) * 255.0,
            ]
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rainbow() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let r = scalar_to_rainbow(&m, "s");
        assert!(r.point_data().get_array("Colors").is_some());
        assert_eq!(
            r.point_data().get_array("Colors").unwrap().num_components(),
            3
        );
    }
    #[test]
    fn test_viridis() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 5.0, 10.0],
                1,
            )));
        let r = scalar_to_viridis(&m, "s");
        assert!(r.point_data().get_array("Colors").is_some());
    }
}
