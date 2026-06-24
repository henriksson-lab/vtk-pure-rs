//! Map scalar data to vertex colors using a colormap.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Map a scalar array to RGB vertex colors using a jet-like colormap.
pub fn scalar_to_vertex_color(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };

    let mut colors = Vec::with_capacity(n * 3);
    for &v in &vals {
        let t = (v - mn) / range;
        let (r, g, b) = jet_color(t);
        colors.push(r * 255.0);
        colors.push(g * 255.0);
        colors.push(b * 255.0);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    result
}

/// Map scalar to grayscale vertex colors.
pub fn scalar_to_grayscale(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };
    let mut colors = Vec::with_capacity(n * 3);
    for &v in &vals {
        let g = ((v - mn) / range * 255.0).clamp(0.0, 255.0);
        colors.push(g);
        colors.push(g);
        colors.push(g);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    result
}

fn jet_color(t: f64) -> (f64, f64, f64) {
    let t = t.clamp(0.0, 1.0);
    let r = (1.5 - (t - 0.75).abs() * 4.0).clamp(0.0, 1.0);
    let g = (1.5 - (t - 0.5).abs() * 4.0).clamp(0.0, 1.0);
    let b = (1.5 - (t - 0.25).abs() * 4.0).clamp(0.0, 1.0);
    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_jet() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "elev",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let r = scalar_to_vertex_color(&mesh, "elev");
        assert!(r.point_data().get_array("Colors").is_some());
        assert_eq!(
            r.point_data().get_array("Colors").unwrap().num_components(),
            3
        );
    }
    #[test]
    fn test_grayscale() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 50.0, 100.0],
                1,
            )));
        let r = scalar_to_grayscale(&mesh, "s");
        let arr = r.point_data().get_array("Colors").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1.0); // min scalar -> dark
    }
}
