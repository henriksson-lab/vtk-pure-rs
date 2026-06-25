use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute gradient direction (angle) of a 2D ImageData field.
///
/// Adds "GradientAngle" scalar in radians (atan2(dy, dx)).
pub fn image_gradient_direction(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let sp = input.spacing();

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let get = |i: i64, j: i64, k: usize| -> f64 {
        values[k * ny * nx
            + (j.clamp(0, ny as i64 - 1) as usize) * nx
            + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    let mut angles = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;
                let gx = (get(ii + 1, jj, k) - get(ii - 1, jj, k)) / (2.0 * sp[0]);
                let gy = (get(ii, jj + 1, k) - get(ii, jj - 1, k)) / (2.0 * sp[1]);
                angles[k * ny * nx + j * nx + i] = gy.atan2(gx);
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientAngle",
            angles,
            1,
        )));
    img
}

/// Compute gradient orientation (0-π, ignoring sign) for edge detection.
pub fn image_gradient_orientation(input: &ImageData, scalars: &str) -> ImageData {
    let result = image_gradient_direction(input, scalars);
    let arr = match result.point_data().get_array("GradientAngle") {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let orient: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let a = buf[0];
            if a < 0.0 {
                a + std::f64::consts::PI
            } else {
                a
            }
        })
        .collect();

    let mut img = result;
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientOrientation",
            orient,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_gradient() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let result = image_gradient_direction(&img, "v");
        let arr = result.point_data().get_array("GradientAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0].abs() < 0.1); // gradient along +X -> angle ~0
    }

    #[test]
    fn vertical_gradient() {
        let mut img = ImageData::with_dimensions(1, 5, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let result = image_gradient_direction(&img, "v");
        let arr = result.point_data().get_array("GradientAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - std::f64::consts::FRAC_PI_2).abs() < 0.1);
    }

    #[test]
    fn orientation_positive() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = image_gradient_orientation(&img, "v");
        let arr = result
            .point_data()
            .get_array("GradientOrientation")
            .unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_gradient_direction(&img, "nope");
        assert!(r.point_data().get_array("GradientAngle").is_none());
    }
}
