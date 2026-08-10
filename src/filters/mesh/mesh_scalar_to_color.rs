//! Map a scalar field to RGB color per vertex using a simple blue-to-red colormap.
use crate::data::{AnyDataArray, DataArray, PolyData};

use crate::filters::mesh::curvature_map_to_color::ColorMapType;

/// Map a scalar field to per-vertex RGB, published as "ColorR"/"ColorG"/"ColorB".
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::mesh::curvature_map_to_color::scalar_to_color`] (a
/// jet/rainbow ramp, the VTK default lookup-table style), splitting its
/// 3-component "Colors" array into the separate channel arrays this entry point
/// has always produced. The combined "Colors" array is kept as well.
pub fn scalar_to_color(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == n => {}
        _ => return mesh.clone(),
    }

    let mut result = crate::filters::mesh::curvature_map_to_color::scalar_to_color(
        mesh,
        scalar_name,
        ColorMapType::Jet,
    );
    let colors = match result.point_data().get_array("Colors") {
        Some(a) => a.to_f64_vec_flat(),
        None => return mesh.clone(),
    };

    let mut r_data = Vec::with_capacity(n);
    let mut g_data = Vec::with_capacity(n);
    let mut b_data = Vec::with_capacity(n);
    for rgb in colors.chunks_exact(3) {
        r_data.push(rgb[0]);
        g_data.push(rgb[1]);
        b_data.push(rgb[2]);
    }

    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("ColorR", r_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("ColorG", g_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("ColorB", b_data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_color() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let r = scalar_to_color(&mesh, "val");
        assert!(r.point_data().get_array("ColorR").is_some());
        assert!(r.point_data().get_array("ColorG").is_some());
        assert!(r.point_data().get_array("ColorB").is_some());
    }

    #[test]
    fn channels_match_combined_colors_array() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let r = scalar_to_color(&mesh, "val");
        let combined = r.point_data().get_array("Colors").unwrap();
        let red = r.point_data().get_array("ColorR").unwrap().to_f64_vec();
        let mut buf = [0.0f64; 3];
        for (i, &channel) in red.iter().enumerate() {
            combined.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - channel).abs() < 1e-12);
        }
    }

    #[test]
    fn rejects_vector_arrays() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 1.0, 0.5, 1.0, 1.0, 0.0],
                2,
            )));

        let r = scalar_to_color(&mesh, "val");
        assert!(r.point_data().get_array("ColorR").is_none());
        assert!(r.point_data().get_array("ColorG").is_none());
        assert!(r.point_data().get_array("ColorB").is_none());
    }
}
