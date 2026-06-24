//! Color mesh by scalar value bands (discrete color levels).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn scalar_bands(mesh: &PolyData, array_name: &str, num_bands: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let nb = num_bands.max(1);
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
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let band = (((v - mn) / range * nb as f64).floor() as usize).min(nb - 1);
            band as f64 / (nb - 1).max(1) as f64
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Bands", data, 1)));
    r.point_data_mut().set_active_scalars("Bands");
    r
}
pub fn scalar_band_colors(mesh: &PolyData, array_name: &str, num_bands: usize) -> PolyData {
    let banded = scalar_bands(mesh, array_name, num_bands);
    let arr = banded.point_data().get_array("Bands").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let palette: [[f64; 3]; 10] = [
        [68.0, 1.0, 84.0],
        [72.0, 35.0, 116.0],
        [64.0, 67.0, 135.0],
        [52.0, 94.0, 141.0],
        [33.0, 145.0, 140.0],
        [53.0, 183.0, 121.0],
        [109.0, 205.0, 89.0],
        [180.0, 222.0, 44.0],
        [229.0, 228.0, 32.0],
        [253.0, 231.0, 37.0],
    ];
    let colors: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let ci = (buf[0] * (palette.len() - 1) as f64).round() as usize;
            let c = palette[ci.min(palette.len() - 1)];
            vec![c[0], c[1], c[2]]
        })
        .collect();
    let mut r = banded;
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bands() {
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
        let r = scalar_bands(&m, "s", 5);
        assert!(r.point_data().get_array("Bands").is_some());
    }
    #[test]
    fn test_colors() {
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
        let r = scalar_band_colors(&m, "s", 5);
        assert!(r.point_data().get_array("Colors").is_some());
    }
}
