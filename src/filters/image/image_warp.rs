//! Image warping and geometric transforms.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Warp image using a displacement field (2-component array).
pub fn warp_by_field(input: &ImageData, scalars: &str, field_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let field = match input.point_data().get_array(field_name) {
        Some(a) if a.num_components() == 2 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut sbuf = [0.0f64];
    let mut fbuf = [0.0f64; 2];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut sbuf);
            sbuf[0]
        })
        .collect();

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let slice = nx * ny;
            let local = idx % slice;
            let iy = local / nx;
            let ix = local % nx;
            field.tuple_as_f64(idx, &mut fbuf);
            let sx = (ix as f64 + fbuf[0]).round() as isize;
            let sy = (iy as f64 + fbuf[1]).round() as isize;
            if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                vals[idx - local + sx as usize + sy as usize * nx]
            } else {
                0.0
            }
        })
        .collect();

    let mut output = ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output.set_extent(input.extent());
    output
}

/// Apply polar coordinate transform (Cartesian to polar).
pub fn cartesian_to_polar(input: &ImageData, scalars: &str) -> ImageData {
    cartesian_to_polar_with_theta_max(input, scalars, 255.0)
}

/// Apply polar coordinate transform with an explicit theta maximum.
pub fn cartesian_to_polar_with_theta_max(
    input: &ImageData,
    scalars: &str,
    theta_max: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() >= 2 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];

    let mut data = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let x = buf[0];
        let y = buf[1];
        let (theta, radius) = if x == 0.0 && y == 0.0 {
            (0.0, 0.0)
        } else {
            let mut theta = y.atan2(x) * theta_max / (2.0 * std::f64::consts::PI);
            if theta < 0.0 {
                theta += theta_max;
            }
            (theta, (x * x + y * y).sqrt())
        };
        data.push(theta);
        data.push(radius);
        for _ in 2..num_components {
            data.push(0.0);
        }
    }

    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            data,
            num_components,
        )));
    output.set_extent(input.extent());
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_polar() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec(
                "v",
                vec![
                    1.0, 0.0, //
                    0.0, 1.0, //
                    0.0, 0.0,
                ],
                2,
            ),
        ));
        let r = cartesian_to_polar_with_theta_max(&img, "v", 360.0);
        assert_eq!(r.dimensions(), [3, 1, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 1.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 90.0).abs() < 1e-12);
        assert_eq!(buf[1], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [0.0, 0.0]);
    }
}
