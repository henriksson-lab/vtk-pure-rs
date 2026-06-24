use crate::data::{AnyDataArray, DataArray, ImageData};

/// Map scalar ImageData through a lookup table to produce RGB colors.
///
/// The lookup table is a list of `[R, G, B]` colors (each in 0.0..1.0).
/// The scalar range is mapped linearly onto the lookup table indices.
///
/// Returns a new ImageData with a 3-component "Colors" array (f64 RGB).
pub fn image_map_to_colors(input: &ImageData, scalars: &str, lut: &[[f64; 3]]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    if n == 0 || lut.is_empty() {
        return input.clone();
    }

    // Find scalar range
    let mut buf = [0.0f64];
    let mut smin = f64::INFINITY;
    let mut smax = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] < smin {
            smin = buf[0];
        }
        if buf[0] > smax {
            smax = buf[0];
        }
    }

    let range = smax - smin;
    let lut_max = (lut.len() - 1) as f64;

    let mut colors = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t = if range.abs() < 1e-15 {
            0.0
        } else {
            ((buf[0] - smin) / range).clamp(0.0, 1.0)
        };
        let fidx = t * lut_max;
        let idx0 = (fidx.floor() as usize).min(lut.len() - 1);
        let idx1 = (idx0 + 1).min(lut.len() - 1);
        let frac = fidx - fidx.floor();

        let c0 = lut[idx0];
        let c1 = lut[idx1];
        colors.push(c0[0] + frac * (c1[0] - c0[0]));
        colors.push(c0[1] + frac * (c1[1] - c0[1]));
        colors.push(c0[2] + frac * (c1[2] - c0[2]));
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_linear_ramp() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.5, 1.0],
                1,
            )));

        let lut = vec![
            [0.0, 0.0, 0.0], // black at min
            [1.0, 1.0, 1.0], // white at max
        ];

        let result = image_map_to_colors(&img, "v", &lut);
        let arr = result.point_data().get_array("Colors").unwrap();

        let mut buf = [0.0f64; 3];

        // First point: scalar 0.0 -> black
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0]).abs() < 1e-10);

        // Middle point: scalar 0.5 -> gray
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);

        // Last point: scalar 1.0 -> white
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn constant_scalar() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0, 5.0],
                1,
            )));

        let lut = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let result = image_map_to_colors(&img, "v", &lut);
        let arr = result.point_data().get_array("Colors").unwrap();

        // Constant scalar -> all map to first color
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10); // Red
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_map_to_colors(&img, "nope", &[[1.0, 0.0, 0.0]]);
        assert!(result.point_data().get_array("Colors").is_none());
    }
}
