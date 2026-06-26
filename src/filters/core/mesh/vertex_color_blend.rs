//! Blend vertex colors between meshes and interpolation utilities.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Blend vertex colors from two arrays on the same mesh.
pub fn blend_vertex_colors(
    mesh: &PolyData,
    array_a: &str,
    array_b: &str,
    alpha: f64,
    output_name: &str,
) -> PolyData {
    let a = match mesh.point_data().get_array(array_a) {
        Some(x) if x.num_components() == 3 => x,
        _ => return mesh.clone(),
    };
    let b = match mesh.point_data().get_array(array_b) {
        Some(x) if x.num_components() == 3 => x,
        _ => return mesh.clone(),
    };
    let n = a.num_tuples().min(b.num_tuples());
    if n != mesh.points.len() || a.num_tuples() != b.num_tuples() {
        return mesh.clone();
    }
    let mut ba = [0.0f64; 3];
    let mut bb = [0.0f64; 3];
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        a.tuple_as_f64(i, &mut ba);
        b.tuple_as_f64(i, &mut bb);
        data.push(alpha * ba[0] + (1.0 - alpha) * bb[0]);
        data.push(alpha * ba[1] + (1.0 - alpha) * bb[1]);
        data.push(alpha * ba[2] + (1.0 - alpha) * bb[2]);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output_name, data, 3)));
    result
}

/// Set uniform color for all vertices.
pub fn set_uniform_color(mesh: &PolyData, r: f64, g: f64, b: f64) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n).flat_map(|_| vec![r, g, b]).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", data, 3)));
    result
}

/// Set vertex color by height (Z coordinate) with linear blue-to-red gradient.
pub fn color_by_height(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let heights: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[2]).collect();
    let mn = heights.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = heights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };
    let data: Vec<f64> = heights
        .iter()
        .flat_map(|&h| {
            let t = (h - mn) / range;
            vec![t * 255.0, 0.0, (1.0 - t) * 255.0]
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", data, 3)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blend() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "A",
                vec![255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0],
                3,
            )));
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "B",
                vec![0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0],
                3,
            )));
        let r = blend_vertex_colors(&mesh, "A", "B", 0.5, "Mixed");
        let arr = r.point_data().get_array("Mixed").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 127.5).abs() < 1e-10);
    }
    #[test]
    fn test_uniform() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = set_uniform_color(&mesh, 128.0, 64.0, 32.0);
        assert!(r.point_data().get_array("Colors").is_some());
    }
    #[test]
    fn test_height() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.5, 1.0, 2.0]],
            vec![[0, 1, 2]],
        );
        let r = color_by_height(&mesh);
        let arr = r.point_data().get_array("Colors").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 200.0); // lowest = blue
    }
}
