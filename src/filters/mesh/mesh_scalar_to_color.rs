//! Map a scalar field to RGB color per vertex using a simple blue-to-red colormap.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_to_color(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0]);
    }
    let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (vmax - vmin).max(1e-15);
    let mut r_data = Vec::with_capacity(n);
    let mut g_data = Vec::with_capacity(n);
    let mut b_data = Vec::with_capacity(n);
    for &v in &vals {
        let t = (v - vmin) / range;
        // Blue -> cyan -> green -> yellow -> red
        let (r, g, b) = if t < 0.25 {
            (0.0, t * 4.0, 1.0)
        } else if t < 0.5 {
            (0.0, 1.0, 1.0 - (t - 0.25) * 4.0)
        } else if t < 0.75 {
            ((t - 0.5) * 4.0, 1.0, 0.0)
        } else {
            (1.0, 1.0 - (t - 0.75) * 4.0, 0.0)
        };
        r_data.push(r);
        g_data.push(g);
        b_data.push(b);
    }
    let mut result = mesh.clone();
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
}
