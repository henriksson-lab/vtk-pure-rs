//! Color mesh vertices by scalar data with configurable colormaps.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Map scalar data to RGB vertex colors using a built-in colormap.
///
/// Colormaps: "jet", "viridis", "grayscale", "blue_red", "cool_warm"
pub fn color_by_scalar(mesh: &PolyData, array_name: &str, colormap: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);

    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t = (buf[0] - min_v) / range;
        let (r, g, b) = apply_colormap(t, colormap);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    result
}

/// Color by height (Z coordinate).
pub fn color_by_height(mesh: &PolyData, colormap: &str) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;
    for i in 0..n {
        let z = mesh.points.get(i)[2];
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }
    let range = (max_z - min_z).max(1e-15);
    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        let t = (mesh.points.get(i)[2] - min_z) / range;
        let (r, g, b) = apply_colormap(t, colormap);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    result
}

/// Color each cell with a unique color based on cell index.
pub fn color_cells_unique(mesh: &PolyData) -> PolyData {
    let n = mesh.polys.num_cells();
    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        let hue = i as f64 / n.max(1) as f64;
        let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.9);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    result
}

fn apply_colormap(t: f64, name: &str) -> (f64, f64, f64) {
    let t = t.clamp(0.0, 1.0);
    match name {
        "jet" => {
            let r = (1.5 - (t - 0.75).abs() * 4.0).clamp(0.0, 1.0);
            let g = (1.5 - (t - 0.5).abs() * 4.0).clamp(0.0, 1.0);
            let b = (1.5 - (t - 0.25).abs() * 4.0).clamp(0.0, 1.0);
            (r, g, b)
        }
        "viridis" => {
            let r = (0.267 + t * 0.329 + t * t * 0.7).clamp(0.0, 1.0);
            let g = (0.004 + t * 0.873).clamp(0.0, 1.0);
            let b = (0.329 + t * 0.198 - t * t * 0.6).clamp(0.0, 1.0);
            (r, g, b)
        }
        "grayscale" => (t, t, t),
        "blue_red" => (t, 0.0, 1.0 - t),
        "cool_warm" => {
            let r = (0.23 + t * 0.77).clamp(0.0, 1.0);
            let g = (0.3 + (0.5 - (t - 0.5).abs()) * 0.8).clamp(0.0, 1.0);
            let b = (1.0 - t * 0.77).clamp(0.0, 1.0);
            (r, g, b)
        }
        _ => (t, t, t),
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_color() {
        let mut m = PolyData::from_points(vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let result = color_by_scalar(&m, "val", "jet");
        let arr = result.point_data().get_array("RGB").unwrap();
        assert_eq!(arr.num_components(), 3);
        assert_eq!(arr.num_tuples(), 3);
    }
    #[test]
    fn height_color() {
        let m = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]]);
        let result = color_by_height(&m, "viridis");
        assert!(result.point_data().get_array("RGB").is_some());
    }
    #[test]
    fn cell_colors() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = color_cells_unique(&m);
        assert!(result.cell_data().get_array("RGB").is_some());
    }
}
